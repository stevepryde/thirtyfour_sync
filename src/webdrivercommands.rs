use std::{fs::File, io::Write, path::Path, time::Duration};

use base64::decode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};

use crate::error::WebDriverError;
use crate::http::connection_sync::WebDriverHttpClientSync;
use crate::WebDriverSession;
use crate::{
    action_chain::ActionChain,
    common::{
        command::Command,
        command::ExtensionCommand,
        connection_common::{convert_json, convert_json_vec},
    },
    error::WebDriverResult,
    webelement::{convert_element_sync, convert_elements_sync},
    By, Cookie, OptionRect, Rect, ScriptArgs, SessionId, SwitchTo, TimeoutConfiguration,
    WebElement, WindowHandle,
};
use thirtyfour::common::command::FormatRequestData;

pub fn start_session<C>(
    conn: &dyn WebDriverHttpClientSync,
    capabilities: C,
) -> WebDriverResult<(SessionId, serde_json::Value)>
where
    C: Serialize,
{
    let caps = serde_json::to_value(capabilities)?;
    let v = match conn.execute(Command::NewSession(caps.clone()).format_request(&SessionId::null()))
    {
        Ok(x) => Ok(x),
        Err(e) => {
            // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
            // Retry if we get a 500. If it happens twice in a row then the second error
            // will be returned.
            if let WebDriverError::UnknownError(x) = &e {
                if x.status == 500 {
                    conn.execute(Command::NewSession(caps).format_request(&SessionId::null()))
                } else {
                    Err(e)
                }
            } else {
                Err(e)
            }
        }
    }?;

    #[derive(Debug, Deserialize)]
    struct ConnectionData {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        #[serde(default)]
        capabilities: Value,
    }

    #[derive(Debug, Deserialize)]
    struct ConnectionResp {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        value: ConnectionData,
    }

    let resp: ConnectionResp = serde_json::from_value(v)?;
    let data = resp.value;
    let session_id = SessionId::from(if resp.session_id.is_empty() {
        data.session_id
    } else {
        resp.session_id
    });
    // Set default timeouts.
    conn.execute(
        Command::SetTimeouts(TimeoutConfiguration::default()).format_request(&session_id),
    )?;

    Ok((session_id, data.capabilities))
}

/// All browser-level W3C WebDriver commands are implemented under this trait.
///
/// `Thirtyfour` is structured as follows:
/// - The `WebDriverCommands` trait contains all of the methods you would
///   typically call in order to interact with the browser.
/// - The `GenericWebDriver` struct implements the `WebDriverCommands` trait
///   for a generic HTTP client.
/// - The `WebDriver` struct is the `GenericWebDriver` implemented for a
///   specific HTTP client.
///
/// You only need to use `WebDriver` in your code. Just create an instance
/// of the `WebDriver` struct and it will have access to all of the methods
/// from the `WebDriverCommands` trait.
///
/// For example:
/// ```rust
/// # use thirtyfour_sync::prelude::*;
/// # fn main() -> WebDriverResult<()> {
/// let caps = DesiredCapabilities::chrome();
/// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// driver.get("http://webappdemo")?;
/// assert_eq!(driver.current_url()?, "http://webappdemo/");
/// #     Ok(())
/// # }
/// ```
pub trait WebDriverCommands {
    /// Get the current session and http client.
    ///
    /// For `thirtyfour` internal use only.
    fn session(&self) -> &WebDriverSession;

    /// Convenience wrapper for running WebDriver commands.
    ///
    /// For `thirtyfour` internal use only.
    fn cmd(&self, command: Command) -> WebDriverResult<serde_json::Value> {
        self.session().execute(Box::new(command))
    }

    /// Close the current window or tab.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo")?;
    /// // Close the tab. This will return to the original tab.
    /// driver.close()?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn close(&self) -> WebDriverResult<()> {
        self.cmd(Command::CloseWindow).map(|_| ())
    }

    /// Navigate to the specified URL.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// driver.get("http://webappdemo")?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.cmd(Command::NavigateTo(url.into())).map(|_| ())
    }

    /// Get the current URL as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// driver.get("http://webappdemo")?;
    /// let url = driver.current_url()?;
    /// #     assert_eq!(url, "http://webappdemo/");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn current_url(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetCurrentUrl)?;
        convert_json(&v["value"])
    }

    /// Get the page source as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// driver.get("http://webappdemo")?;
    /// let source = driver.page_source()?;
    /// #     assert!(source.starts_with(r#"<html lang="en">"#));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn page_source(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetPageSource)?;
        convert_json(&v["value"])
    }

    /// Get the page title as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// driver.get("http://webappdemo")?;
    /// let title = driver.title()?;
    /// #     assert_eq!(title, "Demo Web App");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn title(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetTitle)?;
        convert_json(&v["value"])
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem_text = driver.find_element(By::Name("input1"))?;
    /// let elem_button = driver.find_element(By::Id("button-set"))?;
    /// let elem_result = driver.find_element(By::Id("input-result"))?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.cmd(Command::FindElement(by.into()))?;
        convert_element_sync(self.session(), &v["value"])
    }

    /// Search for all elements on the current page that match the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elems = driver.find_elements(By::ClassName("section"))?;
    /// for elem in elems {
    ///     assert!(elem.get_attribute("class")?.expect("Missing class on element").contains("section"));
    /// }
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.cmd(Command::FindElements(by.into()))?;
        convert_elements_sync(self.session(), &v["value"])
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Use find_element() to wait for the page to load.
    /// #     driver.find_element(By::Id("button1"))?;
    /// let ret = driver.execute_script(r#"
    ///     let elem = document.getElementById("button1");
    ///     elem.click();
    ///     return elem;
    ///     "#
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text()?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// assert_eq!(elem.text()?, "Button 1 clicked");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_script(&self, script: &str) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), Vec::new()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let mut args = ScriptArgs::new();
    /// args.push(elem.clone())?;
    /// args.push("TESTING")?;
    /// let ret = driver.execute_script_with_args(r#"
    ///     arguments[0].innerHTML = arguments[1];
    ///     return arguments[0];
    ///     "#, &args
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text()?, "TESTING");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), args.get_args()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Use find_element() to wait for the page to load.
    /// #     driver.find_element(By::Id("button1"))?;
    /// let ret = driver.execute_async_script(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[0];
    ///     window.setTimeout(() => {
    ///         let elem = document.getElementById("button1");
    ///         elem.click();
    ///         done(elem);
    ///     }, 1000);
    ///     "#
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text()?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// assert_eq!(elem.text()?, "Button 1 clicked");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_async_script(&self, script: &str) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), Vec::new()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let mut args = ScriptArgs::new();
    /// args.push(elem.clone())?;
    /// args.push("TESTING")?;
    /// let ret = driver.execute_async_script_with_args(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[2];
    ///     window.setTimeout(() => {
    ///         arguments[0].innerHTML = arguments[1];
    ///         done(arguments[0]);
    ///     }, 1000);
    ///     "#, &args
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text()?, "TESTING");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_async_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), args.get_args()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Get the current window handle.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle()?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo")?;
    /// assert_ne!(driver.current_window_handle()?, handle);
    /// // Switch back to original tab.
    /// driver.switch_to().window(&handle)?;
    /// assert_eq!(driver.current_window_handle()?, handle);
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self.cmd(Command::GetWindowHandle)?;
        convert_json::<String>(&v["value"]).map(WindowHandle::from)
    }

    /// Get all window handles for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// assert_eq!(driver.window_handles()?.len(), 1);
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// assert_eq!(handles.len(), 2);
    /// driver.switch_to().window(&handles[1])?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self.cmd(Command::GetWindowHandles)?;
        let strings: Vec<String> = convert_json_vec(&v["value"])?;
        Ok(strings.iter().map(WindowHandle::from).collect())
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.maximize_window()?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn maximize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MaximizeWindow).map(|_| ())
    }

    /// Minimize the current window.
    ///
    /// # Example:
    /// ```ignore
    /// # // Minimize is not currently working on Chrome, but does work
    /// # // on Firefox/geckodriver.
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.minimize_window()?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn minimize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MinimizeWindow).map(|_| ())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.fullscreen_window()?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::FullscreenWindow).map(|_| ())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let option_rect = OptionRect::new().with_pos(1, 1).with_size(800, 600);
    /// driver.set_window_rect(option_rect.clone())?;
    /// let rect = driver.get_window_rect()?;
    /// assert_eq!(OptionRect::from(rect), option_rect);
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let v = self.cmd(Command::GetWindowRect)?;
        convert_json(&v["value"])
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// This requires an OptionRect, which is similar to Rect except all
    /// members are wrapped in Option.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::OptionRect;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let r = OptionRect::new().with_size(1280, 720);
    /// driver.set_window_rect(r)?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also convert from a Rect if you want to get the window size
    /// and modify it before setting it again.
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::OptionRect;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let rect = driver.get_window_rect()?;
    /// let option_rect = OptionRect::from(rect);
    /// driver.set_window_rect(option_rect.with_width(1024))?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.cmd(Command::SetWindowRect(rect)).map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// driver.back()?;
    /// #     assert_eq!(driver.title()?, "");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn back(&self) -> WebDriverResult<()> {
        self.cmd(Command::Back).map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     driver.back()?;
    /// #     assert_eq!(driver.title()?, "");
    /// driver.forward()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn forward(&self) -> WebDriverResult<()> {
        self.cmd(Command::Forward).map(|_| ())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// driver.refresh()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn refresh(&self) -> WebDriverResult<()> {
        self.cmd(Command::Refresh).map(|_| ())
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    ///
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     let set_timeouts = TimeoutConfiguration::new(
    /// #         Some(Duration::new(1, 0)),
    /// #         Some(Duration::new(2, 0)),
    /// #         Some(Duration::new(3, 0))
    /// #     );
    /// #     driver.set_timeouts(set_timeouts.clone())?;
    /// let timeouts = driver.get_timeouts()?;
    /// println!("Page load timeout = {:?}", timeouts.page_load());
    /// #     assert_eq!(timeouts.script(), Some(Duration::new(1, 0)));
    /// #     assert_eq!(timeouts.page_load(), Some(Duration::new(2, 0)));
    /// #     assert_eq!(timeouts.implicit(), Some(Duration::new(3, 0)));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
        let v = self.cmd(Command::GetTimeouts)?;
        convert_json(&v["value"])
    }

    /// Set all timeouts for the current session.
    ///
    /// NOTE: If you set timeouts to values greater than 120 seconds,
    ///       remember to also increase the request timeout.
    ///       See `WebDriver::set_request_timeout()` for more details.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    ///
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// // Setting timeouts to None means those timeout values will not be updated.
    /// let timeouts = TimeoutConfiguration::new(None, Some(Duration::new(11, 0)), None);
    /// driver.set_timeouts(timeouts.clone())?;
    /// #     let got_timeouts = driver.get_timeouts()?;
    /// #     assert_eq!(got_timeouts.page_load(), Some(Duration::new(11, 0)));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.cmd(Command::SetTimeouts(timeouts)).map(|_| ())
    }

    /// Set the implicit wait timeout. This is how long the WebDriver will
    /// wait when querying elements.
    ///
    /// By default this is set to 30 seconds.
    ///
    /// **NOTE:** Depending on the kind of testing you want to do, you may
    /// find it more reliable to set the implicit wait time to 0 (no wait)
    /// and implement your own polling loop outside of `thirtyfour`.
    ///
    /// NOTE: If you set any timeouts to values greater than 120 seconds,
    ///       remember to also increase the request timeout.
    ///       See `WebDriver::set_request_timeout()` for more details.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_implicit_wait_timeout(delay)?;
    /// #     let got_timeouts = driver.get_timeouts()?;
    /// #     assert_eq!(got_timeouts.implicit(), Some(delay));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_implicit_wait_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts)
    }

    /// Set the script timeout. This is how long the WebDriver will wait for a
    /// Javascript script to execute.
    ///
    /// By default this is set to 60 seconds.
    ///
    /// NOTE: If you set any timeouts to values greater than 120 seconds,
    ///       remember to also increase the request timeout.
    ///       See `WebDriver::set_request_timeout()` for more details.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_script_timeout(delay)?;
    /// #     let got_timeouts = driver.get_timeouts()?;
    /// #     assert_eq!(got_timeouts.script(), Some(delay));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts)
    }

    /// Set the page load timeout. This is how long the WebDriver will wait
    /// for the page to finish loading.
    ///
    /// By default this is set to 60 seconds.
    ///
    /// NOTE: If you set any timeouts to values greater than 120 seconds,
    ///       remember to also increase the request timeout.
    ///       See `WebDriver::set_request_timeout()` for more details.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_page_load_timeout(delay)?;
    /// #     let got_timeouts = driver.get_timeouts()?;
    /// #     assert_eq!(got_timeouts.page_load(), Some(delay));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts)
    }

    /// Create a new action chain for this session. Action chains can be used
    /// to simulate more complex user input actions involving key combinations,
    /// mouse movements, mouse click, right-click, and more.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem_text = driver.find_element(By::Name("input1"))?;
    /// let elem_button = driver.find_element(By::Id("button-set"))?;
    ///
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem_text, "thirtyfour")
    ///     .move_to_element_center(&elem_button)
    ///     .click()
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Id("input-result"))?;
    /// #     assert_eq!(elem_result.text()?, "thirtyfour");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.session())
    }

    /// Get all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// let cookies = driver.get_cookies()?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #     assert_eq!(
    /// #         cookies.iter().filter(|x| x.value() == &serde_json::json!("value")).count(), 1);
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self.cmd(Command::GetAllCookies)?;
        convert_json_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// let cookie = driver.get_cookie("key")?;
    /// println!("Got cookie: {}", cookie.value());
    /// #     assert_eq!(cookie.value(), &serde_json::json!("value"));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self.cmd(Command::GetNamedCookie(name.to_string()))?;
        convert_json::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// #     assert!(driver.get_cookie("key").is_ok());
    /// driver.delete_cookie("key")?;
    /// #     assert!(driver.get_cookie("key").is_err());
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        self.cmd(Command::DeleteCookie(name.to_string())).map(|_| ())
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// #     assert!(driver.get_cookie("key").is_ok());
    /// driver.delete_all_cookies()?;
    /// #     assert!(driver.get_cookie("key").is_err());
    /// #     assert!(driver.get_cookies()?.is_empty());
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn delete_all_cookies(&self) -> WebDriverResult<()> {
        self.cmd(Command::DeleteAllCookies).map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let cookie = Cookie::new("key", serde_json::json!("value"));
    /// driver.add_cookie(cookie)?;
    /// #     let got_cookie = driver.get_cookie("key")?;
    /// #     assert_eq!(got_cookie.value(), &serde_json::json!("value"));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.cmd(Command::AddCookie(cookie)).map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeScreenshot)?;
        convert_json(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self.session())
    }

    /// Set the current window name.
    /// Useful for switching between windows/tabs using `driver.switch_to().window_name(name)`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle()?;
    /// driver.set_window_name("main")?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo")?;
    /// assert_ne!(driver.current_window_handle()?, handle);
    /// // Switch back to original tab using window name.
    /// driver.switch_to().window_name("main")?;
    /// assert_eq!(driver.current_window_handle()?, handle);
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_window_name(&self, window_name: &str) -> WebDriverResult<()> {
        self.execute_script(&format!(r#"window.name = "{}""#, window_name))?;
        Ok(())
    }

    /// Running an extension command.
    /// Extension commands are browser specific commands and using browser specific endpoints and
    /// parameters.
    ///
    /// # Example
    /// ```no_run
    /// use serde::Serialize;
    /// use thirtyfour_sync::prelude::*;
    /// use thirtyfour::{ExtensionCommand, RequestMethod};
    ///
    /// #[derive(Serialize)]
    /// pub struct AddonInstallCommand {
    ///    pub path: String,
    ///    pub temporary: Option<bool>,
    /// }
    ///
    /// impl ExtensionCommand for AddonInstallCommand {
    ///    fn parameters_json(&self) -> Option<serde_json::Value> {
    ///        Some(serde_json::to_value(self).unwrap())
    ///    }
    ///    fn method(&self) -> RequestMethod {
    ///        RequestMethod::Post
    ///    }
    ///
    ///    fn endpoint(&self) -> String {
    ///        String::from("/moz/addon/install")
    ///    }
    /// }
    ///
    /// fn main()-> WebDriverResult<()> {
    ///     let caps = DesiredCapabilities::firefox();
    ///     let driver = WebDriver::new("http://localhost:4444", &caps)?;
    ///
    ///     let install_command = AddonInstallCommand {
    ///         path: String::from("/path/to/addon.xpi"),
    ///         temporary: Some(true),
    ///     };
    ///
    ///     let response = driver.extension_command(install_command)?;
    ///
    ///     assert_eq!(response.is_string(), true);
    ///
    ///     Ok(())
    /// }
    ///
    /// ```
    fn extension_command<T: ExtensionCommand + Send + Sync + 'static>(
        &self,
        ext_cmd: T,
    ) -> WebDriverResult<serde_json::Value> {
        let response = self.cmd(Command::ExtensionCommand(Box::new(ext_cmd)))?;

        Ok(response["value"].clone())
    }

    /// Execute the specified function in a new browser tab, closing the tab when complete.
    /// The return value will be that of the supplied function, unless an error occurs while
    /// opening or closing the tab.
    ///
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     // Get the current window handle.
    /// #     let handle = driver.current_window_handle()?;
    /// let window_title = driver.in_new_tab(|| {
    ///     driver.get("https://www.google.com")?;
    ///     driver.title()
    /// })?;
    /// #     assert_eq!(window_title, "Google");
    /// #     assert_eq!(driver.current_window_handle()?, handle);
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn in_new_tab<F, T>(&self, f: F) -> WebDriverResult<T>
    where
        F: FnOnce() -> WebDriverResult<T>,
    {
        let existing_handles = self.window_handles()?;
        let handle = self.current_window_handle()?;

        // Open new tab.
        self.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
        let mut new_handles = self.window_handles()?;
        new_handles.retain(|h| !existing_handles.contains(h));
        if new_handles.len() != 1 {
            return Err(WebDriverError::NotFound(
                "new tab".to_string(),
                "Unable to find window handle for new tab".to_string(),
            ));
        }
        self.switch_to().window(&new_handles[0])?;
        let result = f();

        // Close tab.
        self.execute_script(r#"window.close();"#)?;
        self.switch_to().window(&handle)?;

        result
    }
}

/// Helper struct for getting return values from scripts.
/// See the examples for [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
/// and [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script).
pub struct ScriptRetSync<'a> {
    driver: &'a WebDriverSession,
    value: Value,
}

impl<'a> ScriptRetSync<'a> {
    /// Create a new ScriptRetSync. This is typically done automatically via
    /// [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
    /// or [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script)
    pub fn new(driver: &'a WebDriverSession, value: Value) -> Self {
        ScriptRetSync {
            driver,
            value,
        }
    }

    /// Get the raw JSON value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn convert<T>(&self) -> WebDriverResult<T>
    where
        T: DeserializeOwned,
    {
        let v: T = from_value(self.value.clone())?;
        Ok(v)
    }

    /// Get a single WebElement return value.
    /// Your script must return only a single element for this to work.
    pub fn get_element(&self) -> WebDriverResult<WebElement> {
        convert_element_sync(self.driver, &self.value)
    }

    /// Get a vec of WebElements from the return value.
    /// Your script must return an array of elements for this to work.
    pub fn get_elements(&self) -> WebDriverResult<Vec<WebElement>> {
        convert_elements_sync(self.driver, &self.value)
    }
}
