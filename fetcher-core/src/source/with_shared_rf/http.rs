/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use once_cell::sync::Lazy;
use std::fmt::{Debug, Display};
use std::sync::Mutex;
use url::Url;

use crate::entry::Entry;
use crate::error::source::HttpError;
use crate::error::transform::{HttpError as HttpTransformError, InvalidUrlError};
use crate::sink::Message;
use crate::transform::result::{TransformResult, TransformedEntry, TransformedMessage};

const USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.0) Gecko/20100101 Firefox/96.0";

// option because tls init could've failed and we took out the Err()
static CLIENT: Lazy<Mutex<Option<Result<reqwest::Client, HttpError>>>> = Lazy::new(|| {
	Mutex::new(Some(
		reqwest::ClientBuilder::new()
			.timeout(std::time::Duration::from_secs(30))
			.build()
			.map_err(HttpError::TlsInitFailed),
	))
});

pub struct Http {
	pub(crate) url: Url,
	client: reqwest::Client,
}

#[derive(Clone, Copy, Debug)]
pub enum TransformFromField {
	MessageLink,
	RawContents,
}

impl Http {
	/// Create a new Http client
	///
	/// # Errors
	/// if TLS couldn't be initialized
	///
	/// # Panics
	/// This function may panic if a different thread crashes when calling this function
	pub fn new(url: Url) -> Result<Self, HttpError> {
		let mut client_lock = CLIENT.lock().unwrap();

		// take out the error out of the option if there was an error, otherwise just clone the Client
		let client = match client_lock
			.as_ref()
			.ok_or(HttpError::ClientNotInitialized)?
		{
			Ok(client) => client.clone(),
			Err(_) => {
				let e = client_lock
					.take()
					.expect("Option should be not empty because we have just ok_or()'ed it")
					.unwrap_err();

				return Err(e);
			}
		};

		Ok(Self { url, client })
	}

	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Entry, HttpError> {
		tracing::debug!("Fetching HTTP source");

		tracing::trace!("Making a request to {:?}", self.url.as_str());
		let request = self
			.client
			.get(self.url.as_str())
			.header(reqwest::header::USER_AGENT, USER_AGENT)
			.send()
			.await
			.map_err(|e| HttpError::Get(e, self.url.to_string()))?;

		tracing::trace!("Getting text body of the responce");
		let page = request
			.text()
			.await
			.map_err(|e| HttpError::Get(e, self.url.to_string()))?;

		// tracing::trace!("Done. Body: ----------------------------------------\n{page:?}\n----------------------------------------\n");
		tracing::trace!("Done");

		Ok(Entry {
			raw_contents: Some(page),
			msg: Message {
				link: Some(self.url.clone()),
				..Default::default()
			},
			..Default::default()
		})
	}

	pub async fn transform(
		entry: &Entry,
		from_field: TransformFromField,
	) -> Result<TransformedEntry, HttpTransformError> {
		let link = match from_field {
			TransformFromField::MessageLink => entry.msg.link.clone(),
			TransformFromField::RawContents => entry
				.raw_contents
				.as_ref()
				.map(|s| Url::try_from(s.as_str()).map_err(|e| InvalidUrlError(e, s.to_owned())))
				.transpose()?,
		};
		let link = link.ok_or(HttpTransformError::MissingUrl(from_field))?;

		let Entry {
			raw_contents,
			msg: Message { link, .. },
			..
		} = Self::new(link)?.get().await?;

		Ok(TransformedEntry {
			raw_contents: TransformResult::New(raw_contents),
			msg: TransformedMessage {
				link: TransformResult::New(link),
				..Default::default()
			},
			..Default::default()
		})
	}
}

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.finish_non_exhaustive()
	}
}

impl Display for TransformFromField {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			TransformFromField::MessageLink => "message's link",
			TransformFromField::RawContents => "raw_contents",
		})
	}
}
