/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::auth::google::{Google as GoogleAuth, GoogleOAuth2Error as GoogleAuthError};

/// Authentication type for IMAP
pub enum Auth {
	#[expect(clippy::doc_markdown, reason = "false positive")]
	/// Google OAuth2 with full access to Gmail
	GmailOAuth2(GoogleAuth),
	/// An insecure pure text password
	Password(String),
}

pub(super) struct ImapOAuth2<'a> {
	email: &'a str,
	token: &'a str,
}

impl imap::Authenticator for ImapOAuth2<'_> {
	type Response = String;

	fn process(&self, _challenge: &[u8]) -> Self::Response {
		format!("user={}\x01auth=Bearer {}\x01\x01", self.email, self.token)
	}
}

#[async_trait::async_trait]
pub(super) trait GoogleAuthExt {
	async fn as_imap_oauth2<'a>(
		&'a mut self,
		email: &'a str,
	) -> Result<ImapOAuth2<'a>, GoogleAuthError>;
}

#[async_trait::async_trait]
impl GoogleAuthExt for GoogleAuth {
	async fn as_imap_oauth2<'a>(
		&'a mut self,
		email: &'a str,
	) -> Result<ImapOAuth2<'a>, GoogleAuthError> {
		Ok(ImapOAuth2 {
			email,
			token: self.access_token().await?,
		})
	}
}
