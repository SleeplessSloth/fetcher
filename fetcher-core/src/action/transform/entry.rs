/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`TransformEntry`] trait as well as every type that implement it

pub mod feed;
pub mod html;
pub mod http;
pub mod json;
pub mod print;
pub mod use_as;

use async_trait::async_trait;

use super::{Transform, result::TransformedEntry};
use crate::{
	action::transform::error::{TransformError, TransformErrorKind},
	entry::Entry,
};

use std::fmt::Debug;

// TODO: combine with Transform trait?
/// Transform an entry into one or more entries. This is the type transforms should implement as it includes easier error management
#[async_trait]
pub trait TransformEntry: Debug {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform the `entry` into one or several separate entries
	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err>;
}

#[async_trait]
impl<T> Transform for T
where
	T: TransformEntry + Send + Sync,
{
	async fn transform(&self, old_entry: Entry) -> Result<Vec<Entry>, TransformError> {
		self.transform_entry(old_entry.clone())
			.await
			.map(|vec| {
				vec.into_iter()
					.map(|transformed_entry| transformed_entry.into_entry(&old_entry))
					.collect()
			})
			.map_err(|kind| TransformError {
				kind: kind.into(),
				original_entry: old_entry,
			})
	}
}
