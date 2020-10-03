//! Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.
//!
//! It supports the full W3C WebDriver spec.
//! Tested with Chrome and Firefox although any W3C-compatible WebDriver
//! should work.
//!
//! This crate provides a synchronous (i.e. not async) interface for `thirtyfour`.
//! For async, see the [thirtyfour](https://docs.rs/thirtyfour) crate instead.
//!
//! ## Features
//!
//! - All W3C WebDriver and WebElement methods supported
//! - Create new browser session directly via WebDriver (e.g. chromedriver)
//! - Create new browser session via Selenium Standalone or Grid
//! - Automatically close browser session on drop
//! - Find elements (via all common selectors e.g. Id, Class, CSS, Tag, XPath)
//! - Send keys to elements, including key-combinations
//! - Execute Javascript
//! - Action Chains
//! - Get and set cookies
//! - Switch to frame/window/element/alert
//! - Shadow DOM support
//! - Alert support
//! - Capture / Save screenshot of browser or individual element as PNG
//!
//! ## Examples
//!
//! The following example assumes you have a selenium server running
//! at localhost:4444, and a demo web app running at http://webappdemo
//!
//! You can set these up using docker-compose, as follows:
//!
//! ```ignore
//! docker-compose up -d --build
//! ```
//!
//! The included web app demo is purely for demonstration / unit testing
//! purposes and is not required in order to use this library in other projects.
//!
//!
//! ### Example (synchronous):
//!
//! ```rust
//! use thirtyfour::sync::prelude::*;
//!
//! fn main() -> WebDriverResult<()> {
//!     let caps = DesiredCapabilities::chrome();
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
//!
//!     // Navigate to URL.
//!     driver.get("http://webappdemo")?;
//!
//!     // Navigate to page.
//!     driver.find_element(By::Id("pagetextinput"))?.click()?;
//!
//!     // Find element.
//!     let elem_div = driver.find_element(By::Css("div[data-section='section-input']"))?;
//!
//!     // Find element from element.
//!     let elem_text = elem_div.find_element(By::Name("input1"))?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium")?;
//!
//!     // Click the button.
//!     let elem_button = elem_div.find_element(By::Tag("button"))?;
//!     elem_button.click()?;
//!
//!     // Get text value of element.
//!     let elem_result = driver.find_element(By::Id("input-result"))?;
//!     assert_eq!(elem_result.text()?, "selenium");
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![allow(clippy::needless_doctest_main)]

/// Re-export common types.
pub use thirtyfour::common::{
    self,
    capabilities::{
        chrome::ChromeCapabilities, desiredcapabilities::*, edge::EdgeCapabilities,
        firefox::FirefoxCapabilities, ie::InternetExplorerCapabilities, opera::OperaCapabilities,
        safari::SafariCapabilities,
    },
    command::{By, ExtensionCommand, RequestMethod},
    cookie::Cookie,
    keys::{Keys, TypingData},
    scriptargs::ScriptArgs,
    types::*,
};
pub use thirtyfour::error;
pub use thirtyfour::SessionId;

pub use alert::Alert;
pub use session::WebDriverSession;
pub use switch_to::SwitchTo;
pub use webdriver::GenericWebDriver;
pub use webdriver::WebDriver;
pub use webdrivercommands::WebDriverCommands;
pub use webelement::WebElement;

pub mod prelude {
    pub use crate::alert::Alert;
    pub use crate::error::WebDriverResult;
    pub use crate::switch_to::SwitchTo;
    pub use crate::webdriver::WebDriver;
    pub use crate::webdrivercommands::{ScriptRetSync, WebDriverCommands};
    pub use crate::webelement::WebElement;
    pub use thirtyfour::{By, Cookie, DesiredCapabilities, Keys, ScriptArgs, TypingData};
}

mod action_chain;
mod alert;
pub mod http {
    pub mod connection_sync;
    pub mod reqwest_sync;
}
mod session;
mod switch_to;
mod webdriver;
mod webdrivercommands;
mod webelement;
