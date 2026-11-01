//! How `bb debug` prints the debuggee exit code, with where it came from.

use bb_dap::{ExitSource, ExitTruth};

/// A launched debuggee is the adapter's child, so backback cannot read its
/// status; the code is verified only when the adapter reads the process status.
pub(super) fn launched_exit_line(
    code: Option<i64>,
    signal: Option<i64>,
    source: ExitSource,
) -> String {
    exit_line(bb_dap::launch_exit_truth(code, signal, source))
}

/// The exit code with where it came from.
fn exit_line(truth: ExitTruth) -> String {
    match truth {
        ExitTruth::Verified(code) => format!("code {code}"),
        ExitTruth::Corrected { process, adapter } => {
            format!("code {process} (adapter said {adapter})")
        }
        ExitTruth::Signaled => "ended by a signal".to_owned(),
        ExitTruth::Unverified(code) => format!(
            "code {code} (adapter-reported, unverified; see docs/rnd/NETCOREDBG_EXIT_CODE.md)"
        ),
        ExitTruth::Unknown => "code unknown".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_only_exit_code_is_labelled_unverified() {
        let line: String = exit_line(bb_dap::exit_truth(Some(0), None));
        assert_eq!(
            line,
            "code 0 (adapter-reported, unverified; see docs/rnd/NETCOREDBG_EXIT_CODE.md)"
        );
    }

    #[test]
    fn launched_exit_code_is_plain_only_from_a_process_status_adapter() {
        assert_eq!(
            launched_exit_line(Some(3), None, ExitSource::ProcessStatus),
            "code 3"
        );
        assert_eq!(
            launched_exit_line(Some(1), Some(9), ExitSource::ProcessStatus),
            "ended by a signal"
        );
        assert_eq!(
            launched_exit_line(Some(0), None, ExitSource::AdapterWord),
            "code 0 (adapter-reported, unverified; see docs/rnd/NETCOREDBG_EXIT_CODE.md)"
        );
    }

    #[test]
    fn corrected_exit_code_names_both_values() {
        let truth: ExitTruth = ExitTruth::Corrected {
            process: 5,
            adapter: 0,
        };
        assert_eq!(exit_line(truth), "code 5 (adapter said 0)");
        assert_eq!(exit_line(ExitTruth::Verified(3)), "code 3");
    }
}
