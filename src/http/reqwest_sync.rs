use std::fmt::Debug;

use crate::http::connection_sync::{HttpClientCreateParams, WebDriverHttpClientSync};
use crate::{
    common::connection_common::reqwest_support::build_reqwest_headers,
    error::{WebDriverError, WebDriverResult},
};
use std::time::Duration;
use thirtyfour::{RequestData, RequestMethod};

/// Synchronous connection to the remote WebDriver server.
#[derive(Debug)]
pub struct ReqwestDriverSync {
    url: String,
    client: reqwest::blocking::Client,
    timeout: Duration,
}

impl WebDriverHttpClientSync for ReqwestDriverSync {
    fn create(params: HttpClientCreateParams) -> WebDriverResult<Self> {
        let url = params.server_url.trim_end_matches('/').to_owned();
        let headers = build_reqwest_headers(&url)?;
        Ok(ReqwestDriverSync {
            url,
            client: reqwest::blocking::Client::builder().default_headers(headers).build()?,
            timeout: params.timeout.unwrap_or_else(|| Duration::from_secs(120)),
        })
    }

    /// Set the HTTP client request timeout.
    fn set_request_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Execute the specified command and return the data as serde_json::Value.
    fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value> {
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => self.client.get(&url),
            RequestMethod::Post => self.client.post(&url),
            RequestMethod::Delete => self.client.delete(&url),
        };
        request = request.timeout(self.timeout);

        if let Some(x) = request_data.body {
            request = request.json(&x);
        }

        let resp = request.send()?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp.json()?),
            400..=599 => {
                let status = resp.status().as_u16();
                Err(WebDriverError::parse(status, resp.text()?))
            }
            _ => unreachable!(),
        }
    }
}
