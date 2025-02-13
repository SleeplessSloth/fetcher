/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry_to_msg_map;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap::TapOptional;
use tokio::sync::RwLock;

use super::{
	action::Action,
	external_data::{ExternalDataResult, ProvideExternalData},
	named::{JobName, TaskName},
	read_filter,
	sink::Sink,
	source::Source,
};
use crate::FetcherConfigError;
use fetcher_core::{action::Action as CAction, task::Task as CTask, utils::OptionExt};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	#[serde(rename = "read_filter_type")]
	pub read_filter_kind: Option<read_filter::Kind>,
	pub tag: Option<String>,
	pub source: Option<Source>,
	#[serde(rename = "process")]
	pub actions: Option<Vec<Action>>,
	pub entry_to_msg_map_enabled: Option<bool>,
	pub sink: Option<Sink>,
}

impl Task {
	#[tracing::instrument(level = "debug", skip(self, external))]
	pub fn decode_from_conf<D>(
		self,
		job: &JobName,
		task_name: Option<&TaskName>,
		external: &D,
	) -> Result<CTask, FetcherConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		tracing::trace!("Parsing task config");

		let rf = match self.read_filter_kind {
			Some(expected_rf_type) => {
				match external.read_filter(job, task_name, expected_rf_type) {
					ExternalDataResult::Ok(rf) => Some(Arc::new(RwLock::new(rf))),
					ExternalDataResult::Unavailable => {
						tracing::info!("Read filter is unavailable, skipping");
						None
					}
					ExternalDataResult::Err(e) => return Err(e.into()),
				}
			}
			None => None,
		};

		let actions = self.actions.try_map(|acts| {
			let mut acts = itertools::process_results(
				acts.into_iter()
					.filter_map(|act| act.decode_from_conf(rf.clone(), external).transpose()),
				|i| i.flatten().collect::<Vec<_>>(),
			)?;

			if let Some(sink) = self.sink {
				acts.push(CAction::Sink(sink.decode_from_conf(external)?));
			}

			Ok::<_, FetcherConfigError>(acts)
		})?;

		let entry_to_msg_map_enabled = self
			.entry_to_msg_map_enabled
			.tap_some(|b| {
				// TODO: include task name
				tracing::info!(
					"Overriding entry_to_msg_map_enabled for {} from the default to {}",
					job,
					b
				);
			})
			.unwrap_or_else(|| self.source.as_ref().is_some_and(Source::supports_replies));

		let entry_to_msg_map = if entry_to_msg_map_enabled {
			match external.entry_to_msg_map(job, task_name) {
				ExternalDataResult::Ok(v) => Some(v),
				ExternalDataResult::Unavailable => {
					tracing::info!("Entry to message map is unavailable, skipping...");
					None
				}
				ExternalDataResult::Err(e) => return Err(e.into()),
			}
		} else {
			None
		};

		let tag = match (self.tag, task_name) {
			(Some(tag_override), Some(task_name)) => {
				tracing::debug!(
					"Overriding tag from task name {task_name:?} with {tag_override:?}"
				);
				Some(tag_override)
			}
			(Some(tag), None) => {
				tracing::debug!("Setting custom tag {tag:?}");
				Some(tag)
			}
			(None, Some(task_name)) => {
				tracing::trace!("Using task name as tag");
				Some(task_name.as_str().to_owned())
			}
			(None, None) => None,
		};

		Ok(CTask {
			tag,
			source: self
				.source
				.map(|x| x.decode_from_conf(rf, external))
				.transpose()?,
			actions,
			entry_to_msg_map,
		})
	}
}
