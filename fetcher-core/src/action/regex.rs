/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Regex`] action that can be used as a [`transform`](`crate::action::transform`) or a [`filter`](`crate::action::filter`),
//! depending on the [`Action`] used

pub mod action;

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
use ExtractionResult::{Extracted, Matched, NotMatched};

use std::{borrow::Cow, convert::Infallible};

/// Regex with different action depending on [`action`]. All available regex actions include [`Extract`], [`Find`], [`Replace`]
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Regex<A> {
	/// a compiled regular expression
	pub re: regex::Regex,
	pub action: A,
}

impl<A: Action> Regex<A> {
	/// Creates a new Regex with compiled regular expression `re` and [`action`](`Action`)
	///
	/// # Errors
	/// if the regular expression isn't valid
	pub fn new(re: &str, action: A) -> Result<Self, RegexError> {
		Ok(Self {
			re: regex::Regex::new(re)?,
			action,
		})
	}
}

impl Regex<Extract> {
	/// Extracts capture group "s" (?P<s>) from `text`
	#[must_use]
	pub fn extract<'a>(&self, text: &'a str) -> Option<&'a str> {
		match find(&self.re, text) {
			Extracted(s) => Some(s),
			Matched | NotMatched => None,
		}
	}
}

impl TransformField for Regex<Extract> {
	type Error = RegexError;

	fn transform_field(&self, field: Option<&str>) -> Result<TransformResult<String>, RegexError> {
		let field = match field {
			Some(v) => v,
			None => return Ok(TransformResult::Old(None)),
		};

		let transformed = match self.extract(field) {
			Some(s) => s,
			None if self.action.passthrough_if_not_found => field,
			None => return Err(RegexError::CaptureGroupMissing),
		};

		Ok(TransformResult::New(Some(transformed.to_owned())))
	}
}

impl Filter for Regex<Find> {
	fn filter(&self, entries: &mut Vec<Entry>) {
		entries.retain(|ent| {
			let s = match self.action.in_field {
				Field::Title => ent.msg.title.as_deref().map(Cow::Borrowed),
				Field::Body => ent.msg.body.as_deref().map(Cow::Borrowed),
				Field::Link => ent.msg.link.as_ref().map(|s| Cow::Owned(s.to_string())),
			};

			match s {
				None => false,
				Some(s) => match find(&self.re, &s) {
					Matched | Extracted(_) => true,
					NotMatched => false,
				},
			}
		});
	}
}

impl Regex<Replace> {
	/// Replaces `text` with the re
	#[must_use]
	pub fn replace<'a>(&self, text: &'a str) -> Cow<'a, str> {
		self.re.replace(text, &self.action.with)
	}
}

impl TransformField for Regex<Replace> {
	type Error = Infallible;

	fn transform_field(&self, field: Option<&str>) -> Result<TransformResult<String>, Self::Error> {
		Ok(TransformResult::New(
			field.map(|field| self.replace(field).into_owned()),
		))
	}
}

#[derive(Debug)]
pub(crate) enum ExtractionResult<'a> {
	NotMatched,
	Matched,
	Extracted(&'a str),
}

/// Searches for the regular expression in `text` and returns whether it matched or not. Alternatively it extracts capture group "s" (?P<s>) from text if it's present
pub(crate) fn find<'a>(re: &regex::Regex, text: &'a str) -> ExtractionResult<'a> {
	match re.captures(text) {
		Some(capture_groups) => match capture_groups.name("s") {
			Some(s) => ExtractionResult::Extracted(s.as_str()),
			None => ExtractionResult::Matched,
		},
		None => ExtractionResult::NotMatched,
	}
}

#[allow(clippy::unwrap_used)]
#[allow(unused)]
#[cfg(test)]
mod tests {
	use super::action::*;
	use super::*;

	use assert_matches::assert_matches;

	// #[test]
	// fn replace_id() {
	// 	let re = Regex::new(
	// 		"/s-anzeige/(?:.*)/(?P<s>[0-9]+)-",
	// 		Replace {
	// 			with: "$s".to_owned(),
	// 		},
	// 	)
	// 	.unwrap();

	// 	let s = "/s-anzeige/suche-einen-defekten-ps4-controller/2210607105-279-9346";

	// 	assert_eq!(re.replace(s), "2210607105");
	// }

	/*
	TODO: improve these tests
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
	*/
}
