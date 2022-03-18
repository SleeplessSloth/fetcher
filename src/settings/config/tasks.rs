/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that

use fetcher::{
	config::{self, DataSettings, TemplatesField},
	error::{Error, Result},
	task::{NamedTask, Tasks},
};
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use itertools::Itertools; // for .flatten_ok()
use std::path::PathBuf;

use super::CONFIG_FILE_EXT;
use crate::settings;

pub fn get(settings: &DataSettings) -> Result<Tasks> {
	super::cfg_dirs()?
		.into_iter()
		.map(|mut p| {
			p.push("tasks");
			p
		})
		.map(|d| get_all_from(d, settings))
		.flatten_ok()
		.collect()
}

pub fn get_all_from(tasks_dir: PathBuf, settings: &DataSettings) -> Result<Tasks> {
	let glob_str = format!(
		"{tasks_dir}/**/*.{CONFIG_FILE_EXT}",
		tasks_dir = tasks_dir
			.to_str()
			.expect("Non unicode paths are currently unsupported") // FIXME
	);

	let cfgs = glob::glob(&glob_str).unwrap(); // unwrap NOTE: should be safe if the glob pattern is correct

	cfgs.into_iter()
		.filter_map(|c| match c {
			Ok(v) => task(v, settings).transpose(), // TODO: is that okay?
			Err(e) => Some(Err(Error::InaccessibleConfig(
				e.into_error(),
				tasks_dir.clone(),
			))),
		})
		.collect()
}

pub fn task(path: PathBuf, settings: &DataSettings) -> Result<Option<NamedTask>> {
	fn name(path: &PathBuf) -> Option<String> {
		Some(path.file_stem()?.to_str()?.to_owned())
	}

	let templates: TemplatesField = Figment::new()
		.merge(Yaml::file(&path))
		.extract()
		.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

	let mut conf = Figment::new();

	if let Some(templates) = templates.templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(tmpl_name)?.expect("Template not found"); // FIXME

			tracing::debug!("Using template: {:?}", tmpl.path);

			conf = conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	let task: config::Task = conf
		.merge(Yaml::file(&path))
		.extract()
		.map_err(|e| Error::InvalidConfig(e, path.clone()))?;

	let task = task.parse(&path, settings)?;
	if task.disabled {
		tracing::debug!("Found task {:?} but it's disabled", path);
		return Ok(None);
	}

	Ok(Some(task.into_named_task(
		name(&path).expect("Invalid config name"), // FIXME
		path,
	)))
}
