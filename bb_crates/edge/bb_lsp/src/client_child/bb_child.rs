//! Bounded control of the exact child owned by the connection.

use std::num::NonZeroU32;

use bb_process::OwnedGroup;

use super::{LspClient, LspError};

/// The child's process group, owned so drop or quit ends every descendant.
pub(super) fn own_group(child: &tokio::process::Child) -> Option<OwnedGroup> {
    child
        .id()
        .and_then(NonZeroU32::new)
        .map(OwnedGroup::register)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

impl LspClient {
    /// Stop the child process within the configured deadline.
    ///
    /// # Errors
    /// The process cannot be killed or does not exit in time.
    pub async fn stop_child(&mut self) -> Result<(), LspError> {
        drop(self.group.take());
        tokio::time::timeout(self.timeout, self.child.wait())
            .await
            .map_err(|_: tokio::time::error::Elapsed| LspError::Timeout {
                method: String::from("kill"),
                after: self.timeout,
            })?
            .map(|_status: std::process::ExitStatus| ())
            .map_err(LspError::Spawn)
    }
}
