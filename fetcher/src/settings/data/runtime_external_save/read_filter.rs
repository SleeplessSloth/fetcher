/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TruncatingFileWriter;
use crate::settings::context::StaticContext as Context;
use fetcher_config::jobs::{
	external_data::ExternalDataError,
	named::{JobName, TaskName},
	read_filter::{Kind as ReadFilterKind, ReadFilter as ReadFilterConf},
};
use fetcher_core::read_filter::ReadFilter;

use std::fs;

const READ_DATA_DIR: &str = "read";

#[tracing::instrument(level = "debug", skip(cx))]
pub fn get(
	job: &JobName,
	task: Option<&TaskName>,
	expected_rf_kind: ReadFilterKind,
	cx: Context,
) -> Result<Box<dyn ReadFilter>, ExternalDataError> {
	let path = {
		let mut path = cx.data_path.join(READ_DATA_DIR).join(&**job);

		if let Some(task) = task {
			path.push(&**task);
		}

		path
	};

	match fs::read_to_string(&path) {
		Ok(save_file_rf_raw) if save_file_rf_raw.trim().is_empty() => {
			tracing::trace!("Read filter save file is empty");
		}
		Err(e) => {
			tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");
		}
		Ok(save_file_rf_raw) => {
			let conf: ReadFilterConf =
				serde_json::from_str(&save_file_rf_raw).map_err(|e| (e, &path))?;

			// the old read filter saved on disk is of the same type as the one set in config
			if conf != expected_rf_kind {
				return Err(ExternalDataError::new_rf_incompat_with_path(
					expected_rf_kind,
					conf.to_kind(),
					&path,
				));
			}

			return Ok(conf.decode_from_conf(TruncatingFileWriter::new(path)));
		}
	}

	Ok(expected_rf_kind.new_from_kind(TruncatingFileWriter::new(path)))
}
