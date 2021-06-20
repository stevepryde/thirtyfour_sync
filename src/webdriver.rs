use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use log::error;
use serde::Serialize;
use serde_json::Value;

use crate::common::config::WebDriverConfig;
use crate::http::connection_sync::{HttpClientCreateParams, WebDriverHttpClientSync};
use crate::http::reqwest_sync::ReqwestDriverSync;
use crate::webdrivercommands::{start_session, WebDriverCommands};
use crate::{common::command::Command, error::WebDriverResult, DesiredCapabilities};
use crate::{SessionId, WebDriverSession};
use std::time::Duration;

/// The WebDriver struct represents a browser session.
///
/// For full documentation of all WebDriver methods,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
pub type WebDriver = GenericWebDriver<ReqwestDriverSync>;

/// **NOTE:** For WebDriver method documentation,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
///
/// The `thirtyfour` crate uses a generic struct that implements the
/// `WebDriverCommands` trait. The generic struct is then implemented for
/// a specific HTTP client.
///
/// This `GenericWebDriver` struct encapsulates a synchronous Selenium WebDriver browser
/// session. For the async driver, see [GenericWebDriver](../struct.GenericWebDriver.html).
///
/// # Example:
/// ```rust
/// use thirtyfour_sync::prelude::*;
///
/// fn main() -> WebDriverResult<()> {
///     let caps = DesiredCapabilities::chrome();
///     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
///     driver.get("http://webappdemo")?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct GenericWebDriver<T: WebDriverHttpClientSync> {
    pub session: WebDriverSession,
    capabilities: Value,
    quit_on_drop: bool,
    phantom: PhantomData<T>,
}

impl<T: 'static> GenericWebDriver<T>
where
    T: WebDriverHttpClientSync,
{
    /// The GenericWebDriver struct is not intended to be created directly.
    ///
    /// Instead you would use the WebDriver struct, which wires up the
    /// GenericWebDriver with a HTTP client for making requests to the
    /// WebDriver server.
    ///
    /// Create a new WebDriver as follows:
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new<C>(server_url: &str, capabilities: C) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        Self::new_with_timeout(server_url, capabilities, None)
    }

    /// Creates a new GenericWebDriver just like the `new` function. Allows a
    /// configurable timeout for all HTTP requests including the session creation.
    ///
    /// Create a new WebDriver as follows:
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new_with_timeout("http://localhost:4444/wd/hub", &caps, Some(Duration::from_secs(120)))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new_with_timeout<C>(
        server_url: &str,
        capabilities: C,
        timeout: Option<Duration>,
    ) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        let params = HttpClientCreateParams {
            server_url: server_url.to_string(),
            timeout,
        };
        let conn = T::create(params)?;

        let (session_id, session_capabilities) = start_session(&conn, capabilities)?;

        let driver = GenericWebDriver {
            session: WebDriverSession::new(session_id, Arc::new(Mutex::new(conn))),
            capabilities: session_capabilities,
            quit_on_drop: false,
            phantom: PhantomData,
        };

        Ok(driver)
    }

    /// Return a clone of the capabilities as returned by Selenium.
    pub fn capabilities(&self) -> DesiredCapabilities {
        DesiredCapabilities::new(self.capabilities.clone())
    }

    pub fn session_id(&self) -> &SessionId {
        self.session.session_id()
    }

    pub fn config(&self) -> &WebDriverConfig {
        self.session.config()
    }

    pub fn config_mut(&mut self) -> &mut WebDriverConfig {
        self.session.config_mut()
    }

    /// End the webdriver session.
    pub fn quit(mut self) -> WebDriverResult<()> {
        self.cmd(Command::DeleteSession)?;
        self.quit_on_drop = false;
        Ok(())
    }

    /// Set the request timeout for the HTTP client.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// let caps = DesiredCapabilities::chrome();
    /// let mut driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// driver.set_request_timeout(Duration::from_secs(180))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_request_timeout(&mut self, timeout: Duration) -> WebDriverResult<()> {
        self.session.set_request_timeout(timeout)
    }
}

impl<T> WebDriverCommands for GenericWebDriver<T>
where
    T: WebDriverHttpClientSync,
{
    fn session(&self) -> &WebDriverSession {
        &self.session
    }
}

impl<T> Drop for GenericWebDriver<T>
where
    T: WebDriverHttpClientSync,
{
    /// Close the current session when the WebDriver struct goes out of scope.
    fn drop(&mut self) {
        if self.quit_on_drop && !(self.session.session_id()).is_empty() {
            if let Err(e) = self.cmd(Command::DeleteSession) {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}
