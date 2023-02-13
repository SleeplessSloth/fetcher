/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use crate::Error;
use fetcher_core::action::{
	regex::{
		action::{Extract, Find, Replace},
		Regex as CRegex,
	},
	transform::Transform as CTransform,
	Action as CAction,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Regex {
	pub re: String,
	pub action: Action,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
	Find {
		in_field: Field,
	},
	Extract {
		from_field: Field,
		passthrough_if_not_found: bool,
	},
	Replace {
		in_field: Field,
		with: String,
	},
}

impl Regex {
	pub fn parse(self) -> Result<CAction, Error> {
		let re = &self.re;

		Ok(match self.action {
			Action::Find { in_field } => CAction::Filter(
				CRegex::new(
					re,
					Find {
						in_field: in_field.parse(),
					},
				)?
				.into(),
			),
			Action::Extract {
				from_field: field,
				passthrough_if_not_found,
			} => CTransform::Field {
				field: field.parse(),
				kind: CRegex::new(
					re,
					Extract {
						passthrough_if_not_found,
					},
				)?
				.into(),
			}
			.into(),
			Action::Replace { in_field, with } => CTransform::Field {
				field: in_field.parse(),
				kind: CRegex::new(re, Replace { with })?.into(),
			}
			.into(),
		})
	}
}
