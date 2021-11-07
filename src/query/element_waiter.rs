use crate::error::WebDriverError;
use crate::prelude::WebDriverResult;
use crate::query::conditions::handle_errors;
use crate::query::{conditions, ElementPoller, ElementPollerTicker, ElementPredicate};
use crate::WebElement;
use std::time::Duration;
use stringmatch::Needle;

/// High-level interface for performing explicit waits using the builder pattern.
///
/// # Example:
/// ```rust
/// # use thirtyfour_sync::prelude::*;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     let caps = DesiredCapabilities::chrome();
/// #     let mut driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// #     driver.get("http://webappdemo")?;
/// #     let elem = driver.query(By::Id("button1")).first()?;
/// // Wait until the element is displayed.
/// elem.wait_until().displayed()?;
/// #     assert!(elem.is_displayed()?);
/// #     driver.quit()?;
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ElementWaiter<'a> {
    element: &'a WebElement<'a>,
    poller: ElementPoller,
    message: String,
    ignore_errors: bool,
}

impl<'a> ElementWaiter<'a> {
    fn new(element: &'a WebElement<'a>, poller: ElementPoller) -> Self {
        Self {
            element,
            poller,
            message: String::new(),
            ignore_errors: true,
        }
    }

    /// Use the specified ElementPoller for this ElementWaiter.
    /// This will not affect the default ElementPoller used for other waits.
    pub fn with_poller(mut self, poller: ElementPoller) -> Self {
        self.poller = poller;
        self
    }

    /// Provide a human-readable error message to be returned in the case of timeout.
    pub fn error(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// By default a waiter will ignore any errors that occur while polling for the desired
    /// condition(s). However, this behaviour can be modified so that the waiter will return
    /// early if an error is returned from thirtyfour.
    pub fn ignore_errors(mut self, ignore: bool) -> Self {
        self.ignore_errors = ignore;
        self
    }

    /// Force this ElementWaiter to wait for the specified timeout, polling once
    /// after each interval. This will override the poller for this
    /// ElementWaiter only.
    pub fn wait(self, timeout: Duration, interval: Duration) -> Self {
        self.with_poller(ElementPoller::TimeoutWithInterval(timeout, interval))
    }

    fn run_poller(&self, conditions: Vec<ElementPredicate>) -> WebDriverResult<bool> {
        let mut ticker = ElementPollerTicker::new(self.poller.clone());
        loop {
            let mut conditions_met = true;
            for f in &conditions {
                if !f(self.element)? {
                    conditions_met = false;
                    break;
                }
            }

            if conditions_met {
                return Ok(true);
            }

            if !ticker.tick() {
                return Ok(false);
            }
        }
    }

    fn timeout(self) -> WebDriverResult<()> {
        Err(WebDriverError::Timeout(self.message))
    }

    pub fn condition(self, f: ElementPredicate) -> WebDriverResult<()> {
        match self.run_poller(vec![f])? {
            true => Ok(()),
            false => self.timeout(),
        }
    }

    pub fn conditions(self, conditions: Vec<ElementPredicate>) -> WebDriverResult<()> {
        match self.run_poller(conditions)? {
            true => Ok(()),
            false => self.timeout(),
        }
    }

    pub fn stale(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(Box::new(move |elem| {
            handle_errors(elem.is_present().map(|x| !x), ignore_errors)
        }))
    }

    pub fn displayed(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_displayed(ignore_errors))
    }

    pub fn not_displayed(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_displayed(ignore_errors))
    }

    pub fn selected(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_selected(ignore_errors))
    }

    pub fn not_selected(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_selected(ignore_errors))
    }

    pub fn enabled(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_enabled(ignore_errors))
    }

    pub fn not_enabled(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_enabled(ignore_errors))
    }

    pub fn clickable(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_clickable(ignore_errors))
    }

    pub fn not_clickable(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_clickable(ignore_errors))
    }

    pub fn has_class<N>(self, class_name: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_class(class_name, ignore_errors))
    }

    pub fn lacks_class<N>(self, class_name: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_class(class_name, ignore_errors))
    }

    pub fn has_text<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_text(text, ignore_errors))
    }

    pub fn lacks_text<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_text(text, ignore_errors))
    }

    pub fn has_value<N>(self, value: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_value(value, ignore_errors))
    }

    pub fn lacks_value<N>(self, value: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_value(value, ignore_errors))
    }

    pub fn has_attribute<S, N>(self, attribute_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_attribute(attribute_name, value, ignore_errors))
    }

    pub fn lacks_attribute<S, N>(self, attribute_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_attribute(attribute_name, value, ignore_errors))
    }

    pub fn has_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_attributes(desired_attributes, ignore_errors))
    }

    pub fn lacks_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_attributes(desired_attributes, ignore_errors))
    }

    pub fn has_property<S, N>(self, property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_property(property_name, value, ignore_errors))
    }

    pub fn lacks_property<S, N>(self, property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_property(property_name, value, ignore_errors))
    }

    pub fn has_properties<S, N>(self, desired_properties: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_properties(desired_properties, ignore_errors))
    }

    pub fn lacks_properties<S, N>(self, desired_properties: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_properties(desired_properties, ignore_errors))
    }

    pub fn has_css_property<S, N>(self, css_property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
    }

    pub fn lacks_css_property<S, N>(self, css_property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
    }

    pub fn has_css_properties<S, N>(self, desired_css_properties: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
    }

    pub fn lacks_css_properties<S, N>(
        self,
        desired_css_properties: &[(S, N)],
    ) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
    }
}

/// Trait for enabling the ElementWaiter interface.
pub trait ElementWaitable {
    fn wait_until(&self) -> ElementWaiter;
}

impl ElementWaitable for WebElement<'_> {
    /// Return an ElementWaiter instance for more executing powerful explicit waits.
    ///
    /// This uses the builder pattern to construct explicit waits using one of the
    /// provided predicates. Or you can provide your own custom predicate if desired.
    ///
    /// See [ElementWaiter](query/struct.ElementWaiter.html) for more documentation.
    fn wait_until(&self) -> ElementWaiter {
        let poller: ElementPoller = self.session.config().query_poller.clone();
        ElementWaiter::new(self, poller)
    }
}

#[cfg(test)]
/// This function checks if the public  methods implement Send. It is not intended to be executed.
fn _test_is_send() -> WebDriverResult<()> {
    use crate::prelude::*;

    // Helper methods
    fn is_send<T: Send>() {}
    fn is_send_val<T: Send>(_val: &T) {}

    // Pre values
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", &caps)?;
    let elem = driver.find_element(By::Css(r#"div"#))?;

    // ElementWaitCondition
    is_send_val(&elem.wait_until().stale());
    is_send_val(&elem.wait_until().displayed());
    is_send_val(&elem.wait_until().selected());
    is_send_val(&elem.wait_until().enabled());
    is_send_val(&elem.wait_until().condition(Box::new(|elem| elem.is_enabled().or(Ok(false)))));

    Ok(())
}
