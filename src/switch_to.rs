use crate::webdrivercommands::WebDriverCommands;
use crate::WebDriverSession;
use crate::{
    common::command::Command,
    common::types::WindowHandle,
    error::{WebDriverError, WebDriverResult},
    {webelement::convert_element_sync, Alert, WebElement},
};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo<'a> {
    session: &'a WebDriverSession,
}

impl<'a> SwitchTo<'a> {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(session: &'a WebDriverSession) -> Self {
        SwitchTo {
            session,
        }
    }

    ///Convenience wrapper for executing a WebDriver command.
    fn cmd(&self, command: Command) -> WebDriverResult<serde_json::Value> {
        self.session.cmd(command)
    }

    /// Return the element with focus, or the `<body>` element if nothing has focus.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Wait for page load.
    /// #     driver.find_element(By::Id("button1"))?;
    /// // If no element has focus, active_element() will return the body tag.
    /// let elem = driver.switch_to().active_element()?;
    /// assert_eq!(elem.tag_name()?, "body");
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// // Now let's manually focus an element and try active_element() again.
    /// driver.execute_script(r#"document.getElementsByName("input1")[0].focus();"#)?;
    /// let elem = driver.switch_to().active_element()?;
    /// elem.send_keys("selenium")?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.value()?, Some("selenium".to_string()));
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn active_element(self) -> WebDriverResult<WebElement<'a>> {
        let v = self.cmd(Command::GetActiveElement)?;
        convert_element_sync(self.session, &v["value"])
    }

    /// Return Alert struct for processing the active alert on the page.
    ///
    /// See [Alert](struct.Alert.html) documentation for examples.
    pub fn alert(self) -> Alert<'a> {
        Alert::new(self.session)
    }

    /// Switch to the default frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// driver.switch_to().frame_number(0)?;
    /// // We are now inside the iframe.
    /// driver.find_element(By::Id("button1"))?;
    /// driver.switch_to().default_content()?;
    /// // We are now back in the original window.
    /// #     driver.find_element(By::Id("iframeid1"))?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn default_content(self) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameDefault).map(|_| ())
    }

    /// Switch to an iframe by index. The first iframe on the page has index 0.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// driver.switch_to().frame_number(0)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn frame_number(self, frame_number: u16) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameNumber(frame_number)).map(|_| ())
    }

    /// Switch to the specified iframe element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1"))?;
    /// driver.switch_to().frame_element(&elem_iframe)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn frame_element(self, frame_element: &WebElement) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameElement(frame_element.element_id.clone())).map(|_| ())
    }

    /// Switch to the parent frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1"))?;
    /// driver.switch_to().frame_element(&elem_iframe)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// // Now switch back to the parent frame.
    /// driver.switch_to().parent_frame()?;
    /// // We are now back in the parent document.
    /// #     driver.find_element(By::Id("iframeid1"))?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn parent_frame(self) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToParentFrame).map(|_| ())
    }

    /// Switch to the specified window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("button1"))?;
    /// #     driver.switch_to().window(&handles[0])?;
    /// #     driver.find_element(By::Name("input1"))?;
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn window(self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToWindow(handle.clone())).map(|_| ())
    }

    /// Switch to the window with the specified name. This uses the `window.name` property.
    /// You can set a window name via `WebDriver::set_window_name("someName")?`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour_sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Set main window name so we can switch back easily.
    /// driver.set_window_name("mywindow")?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// assert_eq!(driver.title()?, "");
    /// driver.switch_to().window_name("mywindow")?;
    /// // We are now back in the original tab.
    /// assert_eq!(driver.title()?, "Demo Web App");
    /// #     driver.quit()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn window_name(self, name: &str) -> WebDriverResult<()> {
        let original_handle = self.session.current_window_handle()?;
        let handles = &self.session.window_handles()?;
        for handle in handles {
            self.session.switch_to().window(handle)?;
            let ret = &self.session.execute_script(r#"return window.name;"#)?;
            let current_name: String = ret.convert()?;
            if current_name == name {
                return Ok(());
            }
        }

        self.window(&original_handle)?;
        Err(WebDriverError::NotFound(
            format!("window handle '{}'", name),
            "No windows with the specified handle were found".to_string(),
        ))
    }
}
