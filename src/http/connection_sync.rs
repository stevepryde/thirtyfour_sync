use std::fmt::Debug;

use crate::error::WebDriverResult;
use thirtyfour::RequestData;

/// Trait for executing HTTP requests to selenium/webdriver.
/// As long as you have some struct that implements WebDriverHttpClientSync,
/// you can turn it into a WebDriver like this:
///
/// ```ignore
/// // Assuming MyHttpClient implements WebDriverHttpClientSync.
/// pub type MyWebDriver = GenericWebDriver<MyHttpClient>;
/// ```
pub trait WebDriverHttpClientSync: Debug + Send + Sync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self>
    where
        Self: Sized;

    fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value>;
}
