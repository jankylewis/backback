//! Typed DAP results used by the debugger and probe.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Capabilities returned by `initialize`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// Whether the adapter accepts `gotoTargets` and `goto`.
    #[serde(default)]
    pub supports_goto_targets_request: bool,
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
