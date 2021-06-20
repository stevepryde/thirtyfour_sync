use crate::http::connection_sync::WebDriverHttpClientSync;
use crate::GenericWebDriver;
use std::thread;
use std::time::{Duration, Instant};
pub use thirtyfour::query::ElementPoller;

pub struct ElementPollerTicker {
    timeout: Option<Duration>,
    interval: Option<Duration>,
    min_tries: u32,
    start: Instant,
    cur_tries: u32,
}

impl ElementPollerTicker {
    pub fn new(poller: ElementPoller) -> Self {
        let mut ticker = Self {
            timeout: None,
            interval: None,
            min_tries: 0,
            start: Instant::now(),
            cur_tries: 0,
        };

        match poller {
            ElementPoller::NoWait => {}
            ElementPoller::TimeoutWithInterval(timeout, interval) => {
                ticker.timeout = Some(timeout);
                ticker.interval = Some(interval);
            }
            ElementPoller::NumTriesWithInterval(num_tries, interval) => {
                ticker.interval = Some(interval);
                ticker.min_tries = num_tries;
            }
            ElementPoller::TimeoutWithIntervalAndMinTries(timeout, interval, num_tries) => {
                ticker.timeout = Some(timeout);
                ticker.interval = Some(interval);
                ticker.min_tries = num_tries
            }
        }

        ticker
    }

    pub fn tick(&mut self) -> bool {
        self.cur_tries += 1;

        if self.timeout.filter(|t| &self.start.elapsed() < t).is_none()
            && self.cur_tries >= self.min_tries
        {
            return false;
        }

        if let Some(i) = self.interval {
            // Next poll is due no earlier than this long after the first poll started.
            let minimum_elapsed = i * self.cur_tries;

            // But this much time has elapsed since the first poll started.
            let actual_elapsed = self.start.elapsed();

            if actual_elapsed < minimum_elapsed {
                // So we need to wait this much longer.
                thread::sleep(minimum_elapsed - actual_elapsed);
            }
        }

        true
    }
}

impl<T: 'static> GenericWebDriver<T>
where
    T: WebDriverHttpClientSync,
{
    pub fn set_query_poller(&mut self, poller: ElementPoller) {
        self.config_mut().query_poller = poller;
    }
}
