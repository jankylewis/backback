//! Typed DAP results used by the debugger and probe.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::exit::ExitSource;

/// Capabilities returned by `initialize`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// Whether the adapter accepts `gotoTargets` and `goto`.
    #[serde(default)]
    pub supports_goto_targets_request: bool,
    /// Non-standard, patched netcoredbg only: the launch `exitCode` is the
    /// kernel process status, not a default `0`.
    #[serde(default, rename = "netcoredbgLaunchExitCodeIsProcessStatus")]
    pub launch_exit_code_is_process_status: bool,
}

impl Capabilities {
    /// Where a launched debuggee's `exitCode` comes from with this adapter.
    #[must_use]
    pub fn launch_exit_source(&self) -> ExitSource {
        if self.launch_exit_code_is_process_status {
            ExitSource::ProcessStatus
        } else {
            ExitSource::AdapterWord
        }
    }
}

/// A breakpoint line and optional expression.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    /// One-based source line.
    pub line: u32,
    /// Conditional expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// An unsolicited adapter event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Execution stopped.
    Stopped(Value),
    /// Program or debugger output.
    Output(Value),
    /// Debug session ended.
    Terminated(Value),
    /// Program exited.
    Exited(Value),
    /// Breakpoint status changed.
    Breakpoint(Value),
    /// The adapter stopped at a breakpoint because its condition failed to
    /// evaluate. This is not a real hit.
    ConditionError {
        /// The adapter's error text.
        message: String,
        /// The `stopped` event body.
        stopped: Value,
    },
    /// Another protocol event.
    Other(String, Value),
}

impl Event {
    pub(crate) fn from_parts(name: String, body: Option<Value>) -> Self {
        let data: Value = body.unwrap_or(Value::Null);
        match name.as_str() {
            "stopped" => Self::Stopped(data),
            "output" => Self::Output(data),
            "terminated" => Self::Terminated(data),
            "exited" => Self::Exited(data),
            "breakpoint" => Self::Breakpoint(data),
            _ => Self::Other(name, data),
        }
    }

    /// The non-standard `netcoredbgSignal` of an `exited` event: the signal
    /// that ended the debuggee (patched netcoredbg only).
    #[must_use]
    pub fn exit_signal(&self) -> Option<i64> {
        match self {
            Self::Exited(body) => body.get("netcoredbgSignal").and_then(Value::as_i64),
            Self::Stopped(_)
            | Self::Output(_)
            | Self::Terminated(_)
            | Self::Breakpoint(_)
            | Self::ConditionError { .. }
            | Self::Other(_, _) => None,
        }
    }

    /// The `exitCode` of an `exited` event.
    #[must_use]
    pub fn exit_code(&self) -> Option<i64> {
        match self {
            Self::Exited(body) => body.get("exitCode").and_then(Value::as_i64),
            Self::Stopped(_)
            | Self::Output(_)
            | Self::Terminated(_)
            | Self::Breakpoint(_)
            | Self::ConditionError { .. }
            | Self::Other(_, _) => None,
        }
    }
}

/// One thread returned by `threads`.
#[derive(Debug, Clone, Deserialize)]
pub struct Thread {
    /// Adapter thread ID.
    pub id: i64,
    /// Display name.
    pub name: String,
}

/// One frame returned by `stackTrace`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    /// Adapter frame ID.
    pub id: i64,
    /// Method name.
    pub name: String,
    /// One-based source line.
    pub line: u32,
    /// Source description.
    pub source: Option<Value>,
}

/// A scope returned by `scopes`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    /// Display name.
    pub name: String,
    /// Reference for `variables`.
    pub variables_reference: i64,
}

/// A variable returned by `variables`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    /// Display name.
    pub name: String,
    /// Display value.
    pub value: String,
    /// Reference for children, or zero.
    pub variables_reference: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(body: &str) -> Capabilities {
        serde_json::from_str::<Capabilities>(body).unwrap()
    }

    #[test]
    fn launch_exit_source_follows_the_patched_adapter_capability() {
        let patched: Capabilities = caps(r#"{"netcoredbgLaunchExitCodeIsProcessStatus":true}"#);
        assert_eq!(patched.launch_exit_source(), ExitSource::ProcessStatus);
        let stock: Capabilities = caps(r#"{"supportsGotoTargetsRequest":true}"#);
        assert_eq!(stock.launch_exit_source(), ExitSource::AdapterWord);
        let refused: Capabilities = caps(r#"{"netcoredbgLaunchExitCodeIsProcessStatus":false}"#);
        assert_eq!(refused.launch_exit_source(), ExitSource::AdapterWord);
    }

    #[test]
    fn exited_event_carries_the_patched_adapter_signal() {
        let body: Value = serde_json::json!({"exitCode": 1, "netcoredbgSignal": 9});
        let event: Event = Event::from_parts("exited".to_owned(), Some(body));
        assert_eq!((event.exit_code(), event.exit_signal()), (Some(1), Some(9)));
        let plain: Event = Event::from_parts(
            "exited".to_owned(),
            Some(serde_json::json!({"exitCode": 3})),
        );
        assert_eq!(plain.exit_signal(), None);
        let other: Event = Event::from_parts("terminated".to_owned(), None);
        assert_eq!(other.exit_signal(), None);
    }
}
