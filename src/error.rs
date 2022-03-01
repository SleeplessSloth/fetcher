/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{io, path::PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	// disk io stuff
	#[error("XDG error: {0}")]
	Xdg(#[from] xdg::BaseDirectoriesError),

	#[error("Inaccessible config file: {0}")]
	InaccessibleConfig(io::Error),

	#[error("Inaccessible data file ({1}): {0}")]
	InaccessibleData(io::Error, PathBuf),

	#[error("Corrupted data file ({1}): {0}")]
	CorruptedData(serde_json::error::Error, PathBuf),

	#[error("Error writing into {1}: {0}")]
	Write(io::Error, PathBuf),

	#[error("Invalid config: {0}")]
	InvalidConfig(toml::de::Error),

	// stdin & stdout stuff
	#[error("stdin error: {0}")]
	Stdin(io::Error),
	#[error("stdout error: {0}")]
	Stdout(io::Error),

	// network stuff
	#[error("Network IO error: {0}")]
	Network(#[from] reqwest::Error),

	#[error("Google auth: {0}")]
	GoogleAuth(String),

	#[error("Email auth error: {0}")]
	EmailAuth(imap::Error),

	#[error("Email parse error: {0}")]
	EmailParse(#[from] mailparse::MailParseError),

	#[error("IMAP error: {0}")]
	Email(#[from] imap::Error),

	#[error("Twitter auth error: {0}")]
	TwitterAuth(egg_mode::error::Error),

	#[error("Twitter error: {0}")]
	Twitter(#[from] egg_mode::error::Error),

	#[error("RSS error: {0}")]
	Rss(#[from] rss::Error),

	#[error("Telegram request error: {0}")]
	Telegram(#[from] teloxide::RequestError),

	#[error("Invalid DateTime format: {0}")]
	InvalidDateTimeFormat(#[from] chrono::format::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
