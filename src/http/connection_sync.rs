use std::fmt::Debug;

use crate::error::WebDriverResult;
use std::time::Duration;
use thirtyfour::RequestData;

#[derive(Debug, Clone)]
pub struct HttpClientCreateParams {
    pub server_url: String,
    pub timeout: Option<Duration>,
}

/// Trait for executing HTTP requests to selenium/webdriver.
/// As long as you have some struct that implements WebDriverHttpClientSync,
/// you can turn it into a WebDriver like this:
///
/// ```ignore
/// // Assuming MyHttpClient implements WebDriverHttpClientSync.
/// pub type MyWebDriver = GenericWebDriver<MyHttpClient>;
/// ```
pub trait WebDriverHttpClientSync: Debug + Send + Sync {
    fn create(params: HttpClientCreateParams) -> WebDriverResult<Self>
    where
        Self: Sized;

    fn set_request_timeout(&mut self, timeout: Duration);

    fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value>;
}
