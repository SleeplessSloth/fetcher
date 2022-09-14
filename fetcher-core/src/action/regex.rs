/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;

use std::{borrow::Cow, convert::Infallible};

use self::action::{Action, Extract, Find, Replace};
use super::transform::field::TransformField;
use crate::{
	action::{
		filter::Filter,
		transform::{field::Field, result::TransformResult},
	},
	entry::Entry,
	error::transform::RegexError,
};

#[derive(Debug)]
pub struct Regex<A> {
	pub re: regex::Regex,
	action: A,
}

impl<A: Action> Regex<A> {
	pub fn new(re: &str, action: A) -> Result<Self, RegexError> {
		tracing::trace!("Creating Regex transform with str {:?}", re);
		Ok(Self {
			re: regex::Regex::new(re)?,
			action,
		})
	}
}

impl TransformField for Regex<Extract> {
	type Error = RegexError;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, RegexError> {
		use ExtractionResult::{Extracted, Matched, NotMatched};

		let transformed = match extract(&self.re, field) {
			Extracted(s) => s,
			Matched | NotMatched if self.action.passthrough_if_not_found => field,
			_ => return Err(RegexError::CaptureGroupMissing),
		};

		Ok(TransformResult::New(Some(transformed.to_owned())))
	}
}

impl Filter for Regex<Find> {
	fn filter(&self, entries: &mut Vec<Entry>) {
		use ExtractionResult::{Extracted, Matched, NotMatched};

		entries.retain(|ent| {
			let s = match self.action.in_field {
				Field::Title => ent.msg.title.as_deref(),
				Field::Body => ent.msg.body.as_deref(),
			};

			match s {
				None => false,
				Some(s) => match extract(&self.re, s) {
					Matched | Extracted(_) => true,
					NotMatched => false,
				},
			}
		});
	}
}

impl TransformField for Regex<Replace> {
	type Error = Infallible;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, Self::Error> {
		Ok(TransformResult::New(Some(self.replace(field).into_owned())))
	}
}

impl Regex<Replace> {
	pub(crate) fn replace<'a>(&self, text: &'a str) -> Cow<'a, str> {
		self.re.replace(text, &self.action.with)
	}
}

#[derive(Debug)]
pub(crate) enum ExtractionResult<'a> {
	NotMatched,
	Matched,
	Extracted(&'a str),
}

pub(crate) fn extract<'a>(re: &regex::Regex, text: &'a str) -> ExtractionResult<'a> {
	match re.captures(text) {
		Some(capture_groups) => match capture_groups.name("s") {
			Some(s) => ExtractionResult::Extracted(s.as_str()),
			None => ExtractionResult::Matched,
		},
		None => ExtractionResult::NotMatched,
	}
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
	use super::*;

	use assert_matches::assert_matches;

	#[test]
	fn extract_single() {
		let re = Regex::new(
			"Hello, (?P<s>.*)!",
			Extract {
				passthrough_if_not_found: false,
			},
		)
		.unwrap();
		let s = "Hello, world!";

		assert_matches!(extract(&re.re, s), ExtractionResult::Extracted("world"));
	}

	#[test]
	fn extract_not_found() {
		let re = Regex::new(
			"Hello, (?P<s>.*)!",
			Extract {
				passthrough_if_not_found: false,
			},
		)
		.unwrap();
		let s = "Bad string";

		assert_matches!(extract(&re.re, s), ExtractionResult::NotMatched);
	}
}
