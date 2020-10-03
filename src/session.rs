use crate::common::command::Command;
use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::http::connection_sync::WebDriverHttpClientSync;
use crate::SessionId;
use crate::WebDriverCommands;
use std::sync::Arc;

#[derive(Debug)]
pub struct WebDriverSession {
    session_id: SessionId,
    conn: Arc<dyn WebDriverHttpClientSync>,
    config: WebDriverConfig,
}

impl WebDriverSession {
    pub fn new(session_id: SessionId, conn: Arc<dyn WebDriverHttpClientSync>) -> Self {
        Self {
            session_id,
            conn,
            config: WebDriverConfig::new(),
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn config(&self) -> &WebDriverConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut WebDriverConfig {
        &mut self.config
    }

    pub fn execute(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.conn.execute(&self.session_id, command)
    }
}

impl WebDriverCommands for WebDriverSession {
    fn session(&self) -> &WebDriverSession {
        &self
    }
}
