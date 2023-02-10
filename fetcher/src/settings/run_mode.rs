/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::tasks::ParsedTask;

#[derive(Debug)]
pub enum RunMode {
	Normal { once: bool, dry_run: bool },
	VerifyOnly,
	MarkOldEntriesAsRead,
	Manual { once: bool, task: ParsedTask },
}
