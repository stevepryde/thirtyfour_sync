// This wrapper is a fairly direct port of the Select class from the python
// selenium library at:
// https://github.com/SeleniumHQ/selenium/blob/trunk/py/selenium/webdriver/support/select.py

// Copyright 2021 Stephen Pryde and the thirtyfour_sync contributors
// Derived (and modified) from the Selenium project at https://github.com/SeleniumHQ/selenium.
//
// Copyright 2011-2020 Software Freedom Conservancy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::{no_such_element, WebDriverError, WebDriverResult};
use crate::{By, WebElement};

/// Set the selection state of the specified element.
fn set_selected(element: &WebElement<'_>, select: bool) -> WebDriverResult<()> {
    if element.is_selected()? != select {
        element.click()?;
    }
    Ok(())
}

/// Escape the specified string for use in Css or XPath selector.
pub fn escape_string(value: &str) -> String {
    let contains_single = value.contains('\'');
    let contains_double = value.contains('\"');
    if contains_single && contains_double {
        let mut result = vec![String::from("concat(")];
        for substring in value.split('\"') {
            result.push(format!("\"{}\"", substring));
            result.push(String::from(", '\"', "));
        }
        result.pop();
        if value.ends_with('\"') {
            result.push(String::from(", '\"'"));
        }
        return result.join("") + ")";
    }

    if contains_double {
        format!("'{}'", value)
    } else {
        format!("\"{}\"", value)
    }
}

/// Get the longest word in the specified string.
fn get_longest_token(value: &str) -> &str {
    let mut longest = "";
    for item in value.split(' ') {
        if item.len() > longest.len() {
            longest = item;
        }
    }
    longest
}

/// Convenience wrapper for `<select>` elements.
pub struct SelectElement<'a> {
    element: WebElement<'a>,
    multiple: bool,
}

impl<'a> SelectElement<'a> {
    /// Instantiate a new SelectElement struct. The specified element must be a `<select>` element.
    pub fn new(element: &WebElement<'a>) -> WebDriverResult<SelectElement<'a>> {
        let multiple = element.get_attribute("multiple")?.filter(|x| x != "false").is_some();
        let element = element.clone();
        Ok(SelectElement {
            element,
            multiple,
        })
    }

    /// Return a vec of all options belonging to this select tag.
    pub fn options(&self) -> WebDriverResult<Vec<WebElement>> {
        self.element.find_elements(By::Tag("option"))
    }

    /// Return a vec of all selected options belonging to this select tag.
    pub fn all_selected_options(&self) -> WebDriverResult<Vec<WebElement>> {
        let mut selected = Vec::new();
        for option in self.options()? {
            if option.is_selected()? {
                selected.push(option);
            }
        }
        Ok(selected)
    }

    /// Return the first selected option in this select tag.
    pub fn first_selected_option(&self) -> WebDriverResult<WebElement> {
        for option in self.options()? {
            if option.is_selected()? {
                return Ok(option);
            }
        }
        Err(no_such_element("No options are selected"))
    }

    /// Set selection state for all options.
    fn set_selection_all(&self, select: bool) -> WebDriverResult<()> {
        for option in self.options()? {
            set_selected(&option, select)?;
        }
        Ok(())
    }

    /// Set the selection state of options matching the specified value.
    fn set_selection_by_value(&self, value: &str, select: bool) -> WebDriverResult<()> {
        let selector = format!("option[value={}]", escape_string(value));
        let options = self.element.find_elements(By::Css(&selector))?;
        for option in options {
            set_selected(&option, select)?;
            if !self.multiple {
                break;
            }
        }
        Ok(())
    }

    /// Set the selection state of the option at the specified index. This is done by examining
    /// the "index" attribute of an element and not merely by counting.
    fn set_selection_by_index(&self, index: u32, select: bool) -> WebDriverResult<()> {
        let str_index: String = index.to_string();
        for option in self.options()? {
            if option.get_attribute("index")?.filter(|i| i == &str_index).is_some() {
                set_selected(&option, select)?;
                return Ok(());
            }
        }
        Err(no_such_element(&format!("Could not locate element with index {}", index)))
    }

    /// Set the selection state of options that display text matching the specified text.
    /// That is, when given "Bar" this would select an option like:
    ///
    /// `<option value="foo">Bar</option>`
    fn set_selection_by_visible_text(&self, text: &str, select: bool) -> WebDriverResult<()> {
        let mut xpath = format!(".//option[normalize-space(.) = {}]", escape_string(text));
        let options = match self.element.find_elements(By::XPath(&xpath)) {
            Ok(elems) => elems,
            Err(WebDriverError::NoSuchElement(_)) => Vec::new(),
            Err(e) => return Err(e),
        };

        let mut matched = false;
        for option in &options {
            set_selected(&option, select)?;
            if !self.multiple {
                return Ok(());
            }
            matched = true;
        }

        if options.is_empty() && text.contains(' ') {
            let substring_without_space = get_longest_token(text);
            let candidates = if substring_without_space.is_empty() {
                self.options()?
            } else {
                xpath =
                    format!(".//option[contains(.,{})]", escape_string(substring_without_space));
                self.element.find_elements(By::XPath(&xpath))?
            };
            for candidate in candidates {
                if text == candidate.text()? {
                    set_selected(&candidate, select)?;
                    if !self.multiple {
                        return Ok(());
                    }
                    matched = true;
                }
            }
        }

        if !matched {
            Err(no_such_element(&format!("Could not locate element with visible text: {}", text)))
        } else {
            Ok(())
        }
    }

    /// Set the selection state of options that match the specified XPath condition.
    fn set_selection_by_xpath_condition(
        &self,
        condition: &str,
        select: bool,
    ) -> WebDriverResult<()> {
        let xpath = format!(".//option[{}]", condition);
        let options = self.element.find_elements(By::XPath(&xpath))?;
        if options.is_empty() {
            return Err(no_such_element(&format!(
                "Could not locate element matching XPath condition: {:?}",
                xpath
            )));
        }

        for option in &options {
            set_selected(option, select)?;
            if !self.multiple {
                break;
            }
        }

        Ok(())
    }

    /// Set the selection state of options that display text exactly matching the specified text.
    fn set_selection_by_exact_text(&self, text: &str, select: bool) -> WebDriverResult<()> {
        let condition = format!("text() = {}", escape_string(text));
        self.set_selection_by_xpath_condition(&condition, select)
    }

    /// Set the selection state of options that display text containing the specified substring.
    fn set_selection_by_partial_text(&self, text: &str, select: bool) -> WebDriverResult<()> {
        let condition = format!("contains(text(), {})", escape_string(text));
        self.set_selection_by_xpath_condition(&condition, select)
    }

    /// Select all options for this select tag.
    pub fn select_all(&self) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only select all options of a multi-select");
        self.set_selection_all(true)
    }

    /// Select options matching the specified value.
    pub fn select_by_value(&self, value: &str) -> WebDriverResult<()> {
        self.set_selection_by_value(value, true)
    }

    /// Select the option matching the specified index. This is done by examining
    /// the "index" attribute of an element and not merely by counting.
    pub fn select_by_index(&self, index: u32) -> WebDriverResult<()> {
        self.set_selection_by_index(index, true)
    }

    /// Select options with visible text matching the specified text.
    /// That is, when given "Bar" this would select an option like:
    ///
    /// `<option value="foo">Bar</option>`
    pub fn select_by_visible_text(&self, text: &str) -> WebDriverResult<()> {
        self.set_selection_by_visible_text(text, true)
    }

    /// Select options matching the specified XPath condition.
    /// E.g. The specified condition replaces `{}` in this XPath: `.//option[{}]`
    ///
    /// The following example would match `.//option[starts-with(text(), 'pre')]`:
    /// ```ignore
    /// select_by_xpath_condition("starts-with(text(), 'pre')").await?;
    /// ```
    /// For multi-select, all options matching the condition will be selected.
    /// For single-select, only the first matching option will be selected.
    pub fn select_by_xpath_condition(&self, condition: &str) -> WebDriverResult<()> {
        self.set_selection_by_xpath_condition(condition, true)
    }

    /// Select options with visible text exactly matching the specified text.
    /// For multi-select, all options whose text exactly matches will be selected.
    /// For single-select, only the first exact match will be selected.
    pub fn select_by_exact_text(&self, text: &str) -> WebDriverResult<()> {
        self.set_selection_by_exact_text(text, true)
    }

    /// Select options with visible text partially matching the specified text.
    /// For multi-select, all options whose text contains the specified substring will be selected.
    /// For single-select, only the first option containing the substring will be selected.
    pub fn select_by_partial_text(&self, text: &str) -> WebDriverResult<()> {
        self.set_selection_by_partial_text(text, true)
    }

    /// Deselect all options for this select tag.
    pub fn deselect_all(&self) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect all options of a multi-select");
        self.set_selection_all(false)
    }

    /// Deselect options matching the specified value.
    pub fn deselect_by_value(&self, value: &str) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect options of a multi-select");
        self.set_selection_by_value(value, false)
    }

    /// Deselect the option matching the specified index. This is done by examining
    /// the "index" attribute of an element and not merely by counting.
    pub fn deselect_by_index(&self, index: u32) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect options of a multi-select");
        self.set_selection_by_index(index, false)
    }

    /// Deselect options with visible text matching the specified text.
    /// That is, when given "Bar" this would select an option like:
    ///
    /// `<option value="foo">Bar</option>`
    pub fn deselect_by_visible_text(&self, text: &str) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect options of a multi-select");
        self.set_selection_by_visible_text(text, false)
    }

    /// Deselect options matching the specified XPath condition.
    /// E.g. The specified condition replaces `{}` in this XPath: `.//option[{}]`
    ///
    /// The following example would match `.//option[starts-with(text(), 'pre')]`:
    /// ```ignore
    /// deselect_by_xpath_condition("starts-with(text(), 'pre')").await?;
    /// ```
    /// For multi-select, all options matching the condition will be deselected.
    /// For single-select, only the first matching option will be deselected.
    pub fn deselect_by_xpath_condition(&self, condition: &str) -> WebDriverResult<()> {
        self.set_selection_by_xpath_condition(condition, false)
    }

    /// Deselect all options with visible text exactly matching the specified text.
    pub fn deselect_by_exact_text(&self, text: &str) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect options of a multi-select");
        self.set_selection_by_exact_text(text, false)
    }

    /// Deselect all options with visible text partially matching the specified text.
    pub fn deselect_by_partial_text(&self, text: &str) -> WebDriverResult<()> {
        assert!(self.multiple, "You may only deselect options of a multi-select");
        self.set_selection_by_partial_text(text, false)
    }
}
