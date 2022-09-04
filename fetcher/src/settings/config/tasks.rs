/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that

use figment::{
	providers::{Format, Yaml},
	Figment,
};
use serde::Deserialize;
use std::path::PathBuf;

use super::CONFIG_FILE_EXT;
use crate::error::ConfigError;
use crate::settings;
use crate::task::Task;
use crate::{config, task::Tasks};

#[derive(Deserialize, Debug)]
struct DisabledField {
	disabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct TemplatesField {
	templates: Option<Vec<String>>,
}

// #[tracing::instrument(name = "settings:task", skip(settings))]
#[tracing::instrument]
pub(crate) async fn get_all() -> Result<Tasks, ConfigError> {
	let mut tasks = Tasks::new();
	for dir in super::cfg_dirs()?.into_iter().map(|mut p| {
		p.push("tasks");
		p
	}) {
		tasks.extend(get_all_from(dir).await?);
	}

	Ok(tasks)
}

pub(crate) async fn get_all_from(tasks_dir: PathBuf) -> Result<Tasks, ConfigError> {
	let glob_str = format!(
		"{tasks_dir}/**/*.{CONFIG_FILE_EXT}",
		tasks_dir = tasks_dir.to_str().expect("Path is illegal UTF-8") // .ok_or_else(|| ConfigError::BadPath(tasks_dir.clone()))?
	);

	let cfgs = glob::glob(&glob_str).unwrap(); // unwrap NOTE: should be safe if the glob pattern is correct

	let mut tasks = Tasks::new();
	for cfg in cfgs {
		let cfg = cfg.map_err(|e| ConfigError::Read(e.into_error(), tasks_dir.clone()))?;
		let name = cfg
			.strip_prefix(&tasks_dir)
			.unwrap()
			.with_extension("")
			.to_string_lossy()
			.into_owned();

		get(cfg, &name).await?.map(|task| tasks.insert(name, task));
	}

	Ok(tasks)
}

#[tracing::instrument]
pub(crate) async fn get(path: PathBuf, name: &str) -> Result<Option<Task>, ConfigError> {
	tracing::trace!("Parsing a task from file");

	let task_file = Figment::new().merge(Yaml::file(&path));

	let DisabledField { disabled } = task_file
		.extract()
		.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), path.clone()))?;

	if disabled.unwrap_or(false) {
		tracing::trace!("Task is disabled, skipping...");
		return Ok(None);
	}

	let TemplatesField { templates } = task_file
		.extract()
		.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), path.clone()))?;

	let mut full_conf = Figment::new();

	if let Some(templates) = templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(&tmpl_name)?.ok_or_else(|| {
				ConfigError::TemplateNotFound {
					template: tmpl_name.clone(),
					from_task: name.to_owned(),
				}
			})?;

			tracing::trace!("Using template: {:?}", tmpl.path);

			full_conf = full_conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	let full_conf = full_conf.merge(Yaml::file(&path));

	let task: config::Task = full_conf
		.extract()
		.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), path.clone()))?;

	let task_settings = settings::get_task_settings().await?;
	Ok(Some(task.parse(name, &task_settings).await?))
}
