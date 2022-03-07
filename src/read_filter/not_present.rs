/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::collections::VecDeque;

use super::{Id, Identifiable};

#[derive(Default, Debug)]
pub struct ReadFilterNotPresent {
	pub(crate) read_list: VecDeque<String>,
}

impl ReadFilterNotPresent {
	pub(crate) fn last_read(&self) -> Option<Id> {
		// TODO: why doesn't as_deref() work?
		self.read_list.back().map(|s| s.as_str())
	}

	pub(crate) fn remove_read_from<T: Identifiable>(&self, list: &mut Vec<T>) {
		list.retain(|elem| {
			!self
				.read_list
				.iter()
				.any(|read_elem_id| read_elem_id.as_str() == elem.id())
		});
	}

	pub(crate) fn mark_as_read(&mut self, id: Id) {
		self.read_list.push_back(id.to_owned());
	}
}
