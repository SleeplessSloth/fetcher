/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)] // TODO
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

// TODO: more tests

pub mod auth;
pub mod config;
pub mod entry;
pub mod error;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

use std::time::Duration;
use tokio::time::sleep;

use crate::entry::Entry;
use crate::error::Error;
use crate::error::Result;
use crate::read_filter::ReadFilter;
use crate::source::Source;
use crate::task::Task;

pub async fn run_task(t: &mut Task, mut read_filter: Option<&mut ReadFilter>) -> Result<()> {
	loop {
		tracing::trace!("Running...");

		let fetch = async {
			for entry in t.source.get(read_filter.as_deref()).await? {
				// without the .as_deref, the option is moved instead of borrowed
				#[allow(clippy::needless_option_as_deref)]
				process_entry(t, entry, read_filter.as_deref_mut()).await?;
			}

			Ok::<(), Error>(())
		};

		match fetch.await {
			Ok(_) => (),
			Err(e @ Error::NoConnection(_)) => tracing::warn!("{:?}", color_eyre::eyre::eyre!(e)),
			Err(e) => return Err(e),
		}

		tracing::debug!("Sleeping for {time}m", time = t.refresh);
		sleep(Duration::from_secs(t.refresh * 60 /* secs in a min */)).await;
	}
}

#[tracing::instrument(name = "entry", skip_all, fields(id = entry.id.as_str()))]
async fn process_entry(
	t: &mut Task,
	entry: Entry,
	mut read_filter: Option<&mut ReadFilter>,
) -> Result<()> {
	tracing::trace!("Processing entry: {entry:?}");

	t.sink.send(entry.msg, t.tag.as_deref()).await?;
	match (&mut t.source, &mut read_filter) {
		// Email has custom read filtering and read marking
		(Source::Email(e), None) => e.mark_as_read(&entry.id).await?,
		// delete read_filter save file if it was created for some very strange reason for this source type
		(Source::Email(_), Some(_)) => {
			// read_filter.take().unwrap().delete_from_fs()?;
		}
		(_, Some(f)) => f.mark_as_read(&entry.id).await?,
		_ => unreachable!(),
	}

	Ok::<(), Error>(())
}
