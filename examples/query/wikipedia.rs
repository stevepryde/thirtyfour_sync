//! Requires chromedriver running on port 4444:
//!
//!     chromedriver --port=4444
//!
//! Run as follows:
//!
//!     cargo run --example wikipedia

use thirtyfour_sync::prelude::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", &caps)?;

    // Navigate to https://wikipedia.org.
    driver.get("https://wikipedia.org")?;

    let elem_form = driver.query(By::Id("search-form")).first()?;

    // Find element from element using multiple selectors.
    // Each selector will be executed once per poll iteration.
    // The first element to match will be returned.
    let elem_text = elem_form
        .query(By::Css("thiswont.match"))
        .or(By::Id("searchInput"))
        .desc("search input")
        .first()?;

    // Type in the search terms.
    elem_text.send_keys("selenium")?;

    // Click the search button. Optionally name the element to make error messages more readable.
    let elem_button =
        elem_form.query(By::Css("button[type='submit']")).desc("search button").first()?;
    elem_button.click()?;

    // Wait until the button no longer exists (two different ways).
    elem_button.wait_until().error("Timed out waiting for button to become stale").stale()?;
    driver.query(By::Css("button[type='submit']")).nowait().not_exists()?;

    // Look for header to implicitly wait for the page to load.
    driver.query(By::ClassName("firstHeading")).first()?;
    assert_eq!(driver.title()?, "Selenium - Wikipedia");

    driver.quit()?;

    Ok(())
}
