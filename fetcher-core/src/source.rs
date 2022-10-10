/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Source`]s that can fetch data and create new [`Entries`](`Entry`) out of it
// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod email;
pub mod file;
pub mod http;
pub mod reddit;
pub mod twitter;

pub use self::email::Email;
pub use self::file::File;
pub use self::http::Http;
pub use self::reddit::Reddit;
pub use self::twitter::Twitter;

use crate::entry::Entry;
use crate::error::source::EmailError;
use crate::error::source::Error as SourceError;
use crate::read_filter::ReadFilter;

use std::sync::Arc;
use tokio::sync::RwLock;

/// A source that provides a way to get some data once
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Source {
	/// A [`WithSharedRF`] source and its [`ReadFilter`].
	/// It isn't used in this module but is just kept here to be used externally elsewhere
	/// to avoid using it accidentally with a [`WithCustomReadFilter`](`Source::WithCustomReadFilter`)
	WithSharedReadFilter {
		/// The read filter that can be used externally to filter out read entries after processing them
		rf: Option<Arc<RwLock<ReadFilter>>>,
		/// Refer to [`WithSharedRF`]
		kind: WithSharedRF,
	},
	/// Refer to [`WithCustomRF`]
	WithCustomReadFilter(WithCustomRF),
}

/// A source(s) that uses a built-in [`ReadFilter`](`crate::read_filter::ReadFilter`). Since it doesn't contain any read filtering logic itself, there can be several of those in a single source
/// Always contains a vec with sources of the same type
#[derive(Debug)]
pub struct WithSharedRF(Vec<WithSharedRFKind>);

/// All sources that support a shared [`ReadFilter`](`crate::read_filter::ReadFilter`)
#[derive(Debug)]
pub enum WithSharedRFKind {
	/// Refer to [`File`]
	File(File),
	/// Refer to [`Http`]
	Http(Http),
	/// Refer to [`Twitter`]
	Twitter(Twitter),
	/// Refer to [`Reddit`]
	Reddit(Reddit),
}

/// All sources that don't support a built-in Read Filter and handle filtering logic themselves. They all must provide a way to mark an entry as read.
#[derive(Debug)]
pub enum WithCustomRF {
	/// Refer to [`Email`]
	Email(Email),
}

impl Source {
	/// Get all available entries from the source and run them through the parsers
	///
	/// # Errors
	/// * if there was an error fetching from the source
	/// * if there was an error parsing the just fetched entries
	pub async fn get(
		&mut self,
		// transforms: Option<&[Transform]>,
	) -> Result<Vec<Entry>, SourceError> {
		match self {
			Source::WithSharedReadFilter { kind, .. } => kind.get().await,
			Source::WithCustomReadFilter(x) => x.get().await,
		}
	}
}

impl WithSharedRF {
	/// Create a new sources vec that contains one or several pure sources of the same type
	///
	/// # Errors
	/// * if the source list is empty
	/// * if the several sources that were provided are of different [`WithSharedRFKind`] variants
	pub fn new(sources: Vec<WithSharedRFKind>) -> Result<Self, SourceError> {
		match sources.len() {
			0 => return Err(SourceError::EmptySourceList),
			1 => (),
			// assert that all source types are of the same enum variant
			_ => {
				for variants in sources.windows(2) {
					use std::mem::discriminant as disc;

					if disc(&variants[0]) != disc(&variants[1]) {
						return Err(SourceError::SourceListHasDifferentVariants);
					}
				}
			}
		}

		Ok(Self(sources))
	}

	/// Get all entries from the sources
	///
	/// # Errors
	/// if there was an error fetching from a source
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		use WithSharedRFKind as K;

		let mut entries = Vec::new();

		for s in &mut self.0 {
			entries.extend(match s {
				K::Http(x) => vec![x.get().await?],
				K::Twitter(x) => x.get().await?,
				K::File(x) => vec![x.get().await?],
				K::Reddit(x) => x.get().await?,
			});
		}

		Ok(entries)
	}
}

impl WithCustomRF {
	/// Fetch all entries from the source
	///
	/// # Errors
	/// if there was an error fetching from the source (such as a network connection error or maybe even an authentication error)
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(match self {
			Self::Email(x) => x.get().await?,
		})
	}

	/// Delegate for `mark_as_read()` for each [`WithCustomRF`] variant
	#[allow(clippy::missing_errors_doc)]
	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), SourceError> {
		match self {
			Self::Email(x) => x
				.mark_as_read(id)
				.await
				.map_err(|e| Box::new(EmailError::Imap(e)))?,
		};

		Ok(())
	}

	/// Delegate for `remove_read()` for each [`WithCustomRF`] variant
	#[allow(clippy::ptr_arg)]
	pub fn remove_read(&self, _entries: &mut Vec<Entry>) {
		match self {
			Self::Email(_) => (), // NO-OP, emails should already be unread only when fetching
		}
	}
}

impl TryFrom<Vec<WithSharedRFKind>> for WithSharedRF {
	type Error = SourceError;

	fn try_from(value: Vec<WithSharedRFKind>) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl std::ops::Deref for WithSharedRF {
	type Target = [WithSharedRFKind];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}
