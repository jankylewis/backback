//! How a debug session ended: the adapter process, the debuggee, the exit code.

/// What happened to the adapter process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterExit {
    /// Adapter exit code; `None` when a signal ended it.
    pub code: Option<i32>,
    /// What happened to the debuggee when the adapter went away.
    pub debuggee: Debuggee,
}

/// The debuggee's fate after the adapter exited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Debuggee {
    /// No launch or attach was made.
    None,
    /// The session launched it; no process is left in the adapter's group.
    Ended,
    /// The session attached to it; backback never kills an attached process.
    LeftRunning(u32),
    /// The session launched it, but the cleanup could not be confirmed.
    Unconfirmed,
}

/// The exit code backback reports, with where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitTruth {
    /// The real process status; the adapter agreed or sent nothing.
    Verified(i32),
    /// The real process status; the adapter's `exited` event said otherwise.
    Corrected {
        /// Status from the operating system.
        process: i32,
        /// Value in the adapter's `exited` event.
        adapter: i64,
    },
    /// A signal ended the process, so it has no exit code.
    Signaled,
    /// Only the adapter's value exists; backback does not own the process.
    Unverified(i64),
    /// Neither the process nor the adapter gave a value.
    Unknown,
}

impl ExitTruth {
    /// The code to show the user, when one exists.
    #[must_use]
    pub fn code(self) -> Option<i64> {
        match self {
            Self::Verified(code) | Self::Corrected { process: code, .. } => Some(i64::from(code)),
            Self::Unverified(code) => Some(code),
            Self::Signaled | Self::Unknown => None,
        }
    }
}

/// Cross-check the adapter's `exited` code with the real status of an owned process.
#[must_use]
pub fn exit_truth(adapter: Option<i64>, process: Option<std::process::ExitStatus>) -> ExitTruth {
    match (
        process.map(|status: std::process::ExitStatus| status.code()),
        adapter,
    ) {
        (Some(Some(code)), Some(said)) if i64::from(code) != said => ExitTruth::Corrected {
            process: code,
            adapter: said,
        },
        (Some(Some(code)), Some(_) | None) => ExitTruth::Verified(code),
        (Some(None), Some(_) | None) => ExitTruth::Signaled,
        (None, Some(said)) => ExitTruth::Unverified(said),
        (None, None) => ExitTruth::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    fn exited(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn owned_process_status_wins_over_a_wrong_adapter_code() {
        let truth: ExitTruth = exit_truth(Some(0), Some(exited(5)));
        assert_eq!(
            truth,
            ExitTruth::Corrected {
                process: 5,
                adapter: 0
            }
        );
        assert_eq!(truth.code(), Some(5));
    }

    #[test]
    fn agreeing_or_silent_adapter_gives_a_verified_code() {
        assert_eq!(exit_truth(Some(3), Some(exited(3))), ExitTruth::Verified(3));
        assert_eq!(exit_truth(None, Some(exited(3))), ExitTruth::Verified(3));
    }

    #[test]
    fn adapter_only_code_is_unverified_and_signal_has_no_code() {
        assert_eq!(exit_truth(Some(0), None), ExitTruth::Unverified(0));
        assert_eq!(
            exit_truth(Some(0), Some(ExitStatus::from_raw(9))),
            ExitTruth::Signaled
        );
        assert_eq!(exit_truth(None, None).code(), None);
    }
}
