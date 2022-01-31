// FIXME: use lib error type
use anyhow::Result;
use std::str::FromStr;
use teloxide::Bot;
use toml::{value::Map, Value};

use crate::{
	source::{email::EmailFilters, Email, Source, Rss, Twitter},
	sink::{Telegram, Sink},
};

#[derive(Debug)]
pub struct Config {
	pub name: String,
	pub source: Source,
	pub sink: Sink,
	pub refresh: u64,
}

fn env(s: &str) -> String {
	std::env::var(s).unwrap_or_else(|e| panic!("{s} env not found: {e}"))
}

impl Config {
	pub async fn parse(conf_raw: &str) -> Result<Vec<Self>> {
		let tbl = Value::from_str(conf_raw)?;
		let bot = Bot::new(env("BOT_TOKEN"));

		let mut confs: Vec<Self> = Vec::new();
		// NOTE: should be safe. AFAIK the root of a TOML is always a table
		for (name, table) in tbl.as_table().unwrap() {
			let table = table
				.as_table()
				.unwrap_or_else(|| panic!("{name} does not contain a table"));

			let chat_id = format!("{}_CHAT_ID", name.to_ascii_uppercase());
			let sink = Sink::Telegram(Telegram::new(bot.clone(), env(&chat_id)));
			let source = match table
				.get("type")
				.unwrap_or_else(|| panic!("{name} doesn't contain type field"))
				.as_str()
				.unwrap_or_else(|| panic!("{name}'s type field is not a valid string"))
			{
				"rss" => Self::parse_rss(name, table),
				"twitter" => Self::parse_twitter(name, table).await,
				"email" => Self::parse_email(name, table),
				t => panic!("{t} is not a valid type for {name}"),
			};
			let refresh = table
				.get("refresh")
				.unwrap_or_else(|| panic!("{name} doesn't contain a refresh field"))
				.as_integer()
				.unwrap_or_else(|| panic!("{name}'s refresh field is not a valid integer"))
				as u64; // FIXME: figure out if casting with as can cause problems

			confs.push(Config {
				name: name.clone(),
				source,
				sink,
				refresh,
			});
		}

		Ok(confs)
	}

	fn parse_rss(name: &str, table: &Map<String, Value>) -> Source {
		Rss::new(
			name.to_string(),
			table
				.get("url")
				.unwrap_or_else(|| panic!("{name} doesn't contain url field"))
				.as_str()
				.unwrap_or_else(|| panic!("{name}'s url field is not a valid string"))
				.to_string(),
		)
		.into()
	}

	async fn parse_twitter(name: &str, table: &Map<String, Value>) -> Source {
		let filter = table
			.get("filter")
			.unwrap_or_else(|| panic!("{name} doesn't contain filter field"))
			.as_array()
			.unwrap_or_else(|| panic!("{name}'s filter is not an array"))
			.iter()
			.map(|x| {
				x.as_str()
					.unwrap_or_else(|| panic!("{name}'s filter is not a valid string"))
					.to_string()
			})
			.collect::<Vec<String>>();

		Twitter::new(
			name.to_string(),
			table
				.get("pretty_name")
				.unwrap_or_else(|| panic!("{name} doesn't contain pretty_name field"))
				.as_str()
				.unwrap_or_else(|| panic!("{name}'s pretty_name is not a valid string"))
				.to_string(),
			table
				.get("handle")
				.unwrap_or_else(|| panic!("{name} doesn't contain handle field"))
				.as_str()
				.unwrap_or_else(|| panic!("{name}'s handle is not a valid string"))
				.to_string(),
			env("TWITTER_API_KEY"),
			env("TWITTER_API_KEY_SECRET"),
			filter,
		)
		.await
		.unwrap() // FIXME: use proper errors
		.into()
	}

	fn parse_email(name: &str, table: &Map<String, Value>) -> Source {
		let filters = {
			let filters_table = table
				.get("filters")
				.unwrap_or_else(|| panic!("{name} doesn't contain filters field"))
				.as_table()
				.unwrap_or_else(|| panic!("{name}'s filters is not a valid table"));

			let sender = filters_table
				.get("sender")
				// TODO: move out to a separate local(?) fn
				.map(|x| {
					x.as_str()
						.unwrap_or_else(|| {
							panic!("{name}'s filters sender field is not a valid string")
						})
						.to_string()
				});
			let subjects = filters_table.get("subjects").map(|a| {
				a.as_array()
					.unwrap_or_else(|| panic!("{name}'s filters subject is not an valid array"))
					.iter()
					.map(|s| {
						s.as_str()
							.unwrap_or_else(|| {
								panic!("{name}'s filters subject is not a valid string")
							})
							.to_string()
					})
					.collect::<Vec<_>>()
			});
			let exclude_subjects = filters_table.get("exclude_subjects").map(|a| {
				a.as_array()
					.unwrap_or_else(|| panic!("{name}'s filters subject is not an valid array"))
					.iter()
					.map(|s| {
						s.as_str()
							.unwrap_or_else(|| {
								panic!("{name}'s filters exclude_subject is not a valid string")
							})
							.to_string()
					})
				.collect::<Vec<_>>()
			});

			EmailFilters { sender, subjects, exclude_subjects }
		};

		Email::new(
			name.to_string(),
			table
				.get("imap")
				.unwrap_or_else(|| panic!("{name} doesn't contain imap field"))
				.as_str()
				.unwrap_or_else(|| panic!("{name}'s imap is not a valid string"))
				.to_string(),
			env("EMAIL"),
			env("EMAIL_PASS"),
			filters,
			table
				.get("remove")
				.unwrap_or_else(|| panic!("{name} doesn't contain remove field"))
				.as_bool()
				.unwrap_or_else(|| panic!("{name}'s remove is not a valid bool")),
			Some(
				table
					.get("footer")
					.unwrap_or_else(|| panic!("{name} doesn't contain footer field"))
					.as_str()
					.unwrap_or_else(|| panic!("{name}'s footer is not a valid string"))
					.to_string(),
			),
		)
		.into()
	}
}
