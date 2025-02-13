/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::FetcherConfigError;
use fetcher_core::{
	action::{transform::entry::html::query as c_query, transform::field::Replace as CReplace},
	utils::OptionExt,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")] // deny_unknown_fields not allowed since it's flattened in [`Query`]
pub enum ElementKind {
	Tag(String),
	Class(String),
	#[serde(with = "crate::serde_extentions::tuple")]
	Attr(ElementAttr),
}

#[derive(Clone, Debug)]
pub struct ElementAttr {
	pub name: String,
	pub value: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)] // deny_unknown_fields not allowed since it uses flatten
pub struct ElementQuery {
	#[serde(flatten)]
	pub kind: ElementKind,
	pub ignore: Option<Vec<ElementKind>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemQuery {
	pub query: Vec<ElementQuery>,
}

#[derive(Deserialize, Serialize, Clone, Debug)] // deny_unknown_fields not allowed since it's flattened in [`ElementQuery`]
pub struct ElementDataQuery {
	pub optional: Option<bool>,
	pub query: Vec<ElementQuery>,
	pub data_location: DataLocation,
	pub regex: Option<HtmlQueryRegex>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HtmlQueryRegex {
	pub re: String,
	pub replace_with: String,
}

impl ElementKind {
	#[must_use]
	pub fn decode_from_conf(self) -> c_query::ElementKind {
		use ElementKind::{Attr, Class, Tag};

		match self {
			Tag(val) => c_query::ElementKind::Tag(val),
			Class(val) => c_query::ElementKind::Class(val),
			Attr(ElementAttr { name, value }) => c_query::ElementKind::Attr { name, value },
		}
	}
}

impl DataLocation {
	#[must_use]
	pub fn decode_from_conf(self) -> c_query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => c_query::DataLocation::Text,
			Attr(v) => c_query::DataLocation::Attr(v),
		}
	}
}

impl ElementQuery {
	#[must_use]
	pub fn decode_from_conf(self) -> c_query::ElementQuery {
		c_query::ElementQuery {
			kind: self.kind.decode_from_conf(),
			ignore: self.ignore.map(|v| {
				v.into_iter()
					.map(ElementKind::decode_from_conf)
					.collect::<Vec<_>>()
			}),
		}
	}
}

impl ElementDataQuery {
	pub fn decode_from_conf(self) -> Result<c_query::ElementDataQuery, FetcherConfigError> {
		Ok(c_query::ElementDataQuery {
			optional: self.optional.unwrap_or(false),
			query: self
				.query
				.into_iter()
				.map(ElementQuery::decode_from_conf)
				.collect(),
			data_location: self.data_location.decode_from_conf(),
			regex: self.regex.try_map(HtmlQueryRegex::decode_from_conf)?,
		})
	}
}

impl HtmlQueryRegex {
	pub fn decode_from_conf(self) -> Result<CReplace, FetcherConfigError> {
		CReplace::new(&self.re, self.replace_with).map_err(Into::into)
	}
}

impl<'a> From<&'a ElementAttr> for (&'a String, &'a String) {
	fn from(ElementAttr { name, value }: &'a ElementAttr) -> Self {
		(name, value)
	}
}

impl From<(String, String)> for ElementAttr {
	fn from((name, value): (String, String)) -> Self {
		Self { name, value }
	}
}
