/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::{
	FetcherConfigError as ConfigError,
	jobs::external_data::{ExternalDataResult, ProvideExternalData},
};
use fetcher_core::sink::{Telegram as CTelegram, telegram::LinkLocation as CLinkLocation};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Telegram {
	pub chat_id: i64,
	pub link_location: Option<LinkLocation>,
}

/// Refer to [`crate::sink::message::LinkLocation`]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum LinkLocation {
	PreferTitle,
	Bottom,
}

impl Telegram {
	pub fn decode_from_conf<D>(self, external: &D) -> Result<CTelegram, ConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		let token = match external.telegram_bot_token() {
			ExternalDataResult::Ok(v) => v,
			ExternalDataResult::Unavailable => return Err(ConfigError::TelegramBotTokenMissing),
			ExternalDataResult::Err(e) => return Err(e.into()),
		};

		Ok(CTelegram::new(
			token,
			self.chat_id,
			self.link_location
				.map_or(CLinkLocation::PreferTitle, LinkLocation::decode_from_conf),
		))
	}
}

impl LinkLocation {
	pub fn decode_from_conf(self) -> CLinkLocation {
		match self {
			LinkLocation::PreferTitle => CLinkLocation::PreferTitle,
			LinkLocation::Bottom => CLinkLocation::Bottom,
		}
	}
}
