//! Bounded control of the exact child owned by the connection.

use super::{LspClient, LspError};

impl LspClient {
    /// Stop the child process within the configured deadline.
    ///
    /// # Errors
    /// The process cannot be killed or does not exit in time.
    pub async fn stop_child(&mut self) -> Result<(), LspError> {
        tokio::time::timeout(self.timeout, self.child.kill())
            .await
            .map_err(|_: tokio::time::error::Elapsed| LspError::Timeout {
                method: String::from("kill"),
                after: self.timeout,
            })?
            .map_err(LspError::Spawn)
    }
}
