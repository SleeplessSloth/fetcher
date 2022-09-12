/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TransformField;
use crate::action::transform::result::TransformResult;

use std::convert::Infallible;

#[derive(Debug)]
pub struct Caps;

impl TransformField for Caps {
	type Error = Infallible;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, Infallible> {
		Ok(TransformResult::New(Some(field.to_uppercase())))
	}
}
