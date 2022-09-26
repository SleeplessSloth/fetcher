/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::action::regex as c_regex;
use fetcher_core::action::transform::entry::html::query as c_query;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ElementKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ElementQuery {
	#[serde(flatten)]
	pub kind: ElementKind,
	pub ignore: Option<Vec<ElementKind>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HtmlQueryRegex {
	re: String,
	replace_with: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ElementDataQuery {
	pub optional: Option<bool>,
	pub query: Vec<ElementQuery>,
	pub data_location: DataLocation,
	pub regex: Option<HtmlQueryRegex>,
}

impl ElementKind {
	pub fn parse(self) -> c_query::ElementKind {
		use ElementKind::{Attr, Class, Tag};

		match self {
			Tag(val) => c_query::ElementKind::Tag(val),
			Class(val) => c_query::ElementKind::Class(val),
			Attr { name, value } => c_query::ElementKind::Attr { name, value },
		}
	}
}

impl DataLocation {
	fn parse(self) -> c_query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => c_query::DataLocation::Text,
			Attr(v) => c_query::DataLocation::Attr(v),
		}
	}
}

impl ElementQuery {
	pub fn parse(self) -> c_query::ElementQuery {
		c_query::ElementQuery {
			kind: self.kind.parse(),
			ignore: self
				.ignore
				.map(|v| v.into_iter().map(ElementKind::parse).collect::<Vec<_>>()),
		}
	}
}

impl ElementDataQuery {
	pub fn parse(self) -> Result<c_query::ElementDataQuery, Error> {
		Ok(c_query::ElementDataQuery {
			optional: self.optional.unwrap_or(false),
			query: self.query.into_iter().map(ElementQuery::parse).collect(),
			data_location: self.data_location.parse(),
			regex: self.regex.map(HtmlQueryRegex::parse).transpose()?,
		})
	}
}

impl HtmlQueryRegex {
	pub fn parse(self) -> Result<c_regex::Regex<c_regex::action::Replace>, Error> {
		c_regex::Regex::new(
			&self.re,
			c_regex::action::Replace {
				with: self.replace_with,
			},
		)
		.map_err(Into::into)
	}
}
