use std::fmt::Debug;

use crate::http::connection_sync::{HttpClientCreateParams, WebDriverHttpClientSync};
use crate::{
    common::command::{Command, RequestMethod},
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

/// Null driver that satisfies the build but does nothing.
#[derive(Debug)]
pub struct NullDriverSync {
    url: String,
}

impl WebDriverHttpClientSync for NullDriverSync {
    fn create(params: HttpClientCreateParams) -> WebDriverResult<Self> {
        Ok(NullDriverSync {
            url: params.server_url.to_string(),
        })
    }

    fn set_request_timeout(&mut self, _timeout: Duration) {}

    fn execute(
        &self,
        _session_id: &SessionId,
        _command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
}
