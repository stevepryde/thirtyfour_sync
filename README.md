[![Crates.io](https://img.shields.io/crates/v/thirtyfour_sync.svg?style=for-the-badge)](https://crates.io/crates/thirtyfour_sync)
[![docs.rs](https://img.shields.io/badge/docs.rs-thirtyfour_sync-blue?style=for-the-badge)](https://docs.rs/thirtyfour_sync)
[![Build Status](https://img.shields.io/github/workflow/status/stevepryde/thirtyfour_sync/build-check/main?style=for-the-badge)](https://github.com/stevepryde/thirtyfour_sync/actions)

Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.

It supports the full W3C WebDriver spec. Tested with Chrome and Firefox although any W3C-compatible WebDriver should work.

This crate provides a synchronous (i.e. not async) interface for `thirtyfour`.
For async, see the [thirtyfour](https://docs.rs/thirtyfour) crate instead.

## Features

- All W3C WebDriver and WebElement methods supported
- Create new browser session directly via WebDriver (e.g. chromedriver)
- Create new browser session via Selenium Standalone or Grid
- Automatically close browser session on drop
- Find elements (via all common selectors e.g. Id, Class, CSS, Tag, XPath)
- Send keys to elements, including key-combinations
- Execute Javascript
- Action Chains
- Get and set cookies
- Switch to frame/window/element/alert
- Shadow DOM support
- Alert support
- Capture / Save screenshot of browser or individual element as PNG
- Chrome DevTools Protocol support

## Why 'thirtyfour' ?

It is named after the atomic number for the Selenium chemical element (Se).

## Examples

The example assumes you have a WebDriver running at localhost:4444.

You can use chromedriver directly by downloading the chromedriver that matches your Chrome version,
from here: [https://chromedriver.chromium.org/downloads](https://chromedriver.chromium.org/downloads)

Then run it like this:

    chromedriver --port=4444

### Example:

To run this example:

    cargo run --example sync

```rust
use thirtyfour_sync::prelude::*;

fn main() -> WebDriverResult<()> {
     let caps = DesiredCapabilities::chrome();
     let driver = WebDriver::new("http://localhost:4444", &caps)?;

     // Navigate to https://wikipedia.org.
     driver.get("https://wikipedia.org")?;
     let elem_form = driver.find_element(By::Id("search-form"))?;

     // Find element from element.
     let elem_text = elem_form.find_element(By::Id("searchInput"))?;

     // Type in the search terms.
     elem_text.send_keys("selenium")?;

     // Click the search button.
     let elem_button = elem_form.find_element(By::Css("button[type='submit']"))?;
     elem_button.click()?;

     // Look for header to implicitly wait for the page to load.
     driver.find_element(By::ClassName("firstHeading"))?;
     assert_eq!(driver.title()?, "Selenium - Wikipedia");

     Ok(())
}
```

## Running the tests for `thirtyfour_sync`, including doctests

You generally only need to run the tests if you plan on contributing to the development of `thirtyfour_sync`. If you just want to use the crate in your own project, you can skip this section.

Just like the examples above, the tests in this crate require a running instance of Selenium at `http://localhost:4444`.

The tests also require a small web app called `thirtyfour_testapp` that was purpose-built for testing the `thirtyfour` crate.

This can be run using docker and docker-compose.

To install docker, see [https://docs.docker.com/install/](https://docs.docker.com/install/) (follow the SERVER section if you're on Linux, then look for the Community Edition)

To install docker-compose, see [https://docs.docker.com/compose/install/](https://docs.docker.com/compose/install/)

Once you have docker-compose installed, you can start the required containers, as follows:

    docker-compose up -d --build

Then, to run the tests:

    cargo test -- --test-threads=1

We need to limit the tests to a single thread because the selenium server only supports 1 browser instance at a time.
(You can increase this limit in the `docker-compose.yml` file if you want. Remember to restart the containers afterwards)

If you need to restart the docker containers:

    docker-compose restart 

And finally, to remove them:

    docker-compose down

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

See the NOTICE file for more details.

`SPDX-License-Identifier: MIT OR Apache-2.0`
