//! Inspect stopped threads, frames, scopes, values and the exception a
//! thread stopped on; choose which exceptions stop the program.

use super::DapSession;
use crate::{DapError, Scope, StackFrame, Thread, Variable};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct ThreadsBody {
    threads: Vec<Thread>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FramesBody {
    stack_frames: Vec<StackFrame>,
}
#[derive(Deserialize)]
struct ScopesBody {
    scopes: Vec<Scope>,
}
#[derive(Deserialize)]
struct VariablesBody {
    variables: Vec<Variable>,
}

/// What `exceptionInfo` says about the exception a thread stopped for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionInfo {
    /// The type, e.g. `System.InvalidOperationException`.
    pub exception_id: String,
    /// The message, when the adapter puts it here.
    #[serde(default)]
    pub description: Option<String>,
    /// Nested details; netcoredbg puts the message under `message`.
    #[serde(default)]
    pub details: Option<Value>,
}

impl ExceptionInfo {
    /// `Type: message`, as VS Code's exception widget shows it: the full
    /// type name and the message from `details` when the adapter gives them
    /// (netcoredbg does), else the id without its `CLR/` category and the
    /// description.
    #[must_use]
    pub fn headline(&self) -> String {
        let kind: &str = detail(self.details.as_ref(), "fullTypeName").unwrap_or_else(|| {
            self.exception_id
                .rsplit('/')
                .next()
                .unwrap_or(&self.exception_id)
        });
        let message: Option<&str> = detail(self.details.as_ref(), "message")
            .or_else(|| self.description.as_deref().map(str::trim))
            .filter(|text: &&str| !text.is_empty());
        match message {
            Some(text) => format!("{kind}: {text}"),
            None => kind.to_owned(),
        }
    }
}

/// A non-empty string field of `details`.
fn detail<'a>(details: Option<&'a Value>, key: &str) -> Option<&'a str> {
    details?
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|text: &&str| !text.is_empty())
}

impl DapSession {
    /// List threads.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn threads(&self) -> Result<Vec<Thread>, DapError> {
        Ok(self
            .request::<ThreadsBody>("threads", json!({}))
            .await?
            .threads)
    }

    /// List frames for a stopped thread.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn stack_trace(&self, thread_id: i64) -> Result<Vec<StackFrame>, DapError> {
        Ok(self
            .request::<FramesBody>("stackTrace", json!({"threadId": thread_id}))
            .await?
            .stack_frames)
    }

    /// List scopes for a frame.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn scopes(&self, frame_id: i64) -> Result<Vec<Scope>, DapError> {
        Ok(self
            .request::<ScopesBody>("scopes", json!({"frameId": frame_id}))
            .await?
            .scopes)
    }

    /// List variables in a scope.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn variables(&self, reference: i64) -> Result<Vec<Variable>, DapError> {
        Ok(self
            .request::<VariablesBody>("variables", json!({"variablesReference": reference}))
            .await?
            .variables)
    }

    /// Evaluate an expression in a frame.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn evaluate(&self, expression: &str, frame_id: i64) -> Result<Value, DapError> {
        self.request(
            "evaluate",
            json!({"expression": expression, "frameId": frame_id, "context": "watch"}),
        )
        .await
    }

    /// Break on the exceptions the adapter's `filters` name (an empty list
    /// breaks on none).
    ///
    /// # Errors
    /// DAP request error.
    pub async fn set_exception_breakpoints(&self, filters: &[String]) -> Result<(), DapError> {
        let _: Value = self
            .request("setExceptionBreakpoints", json!({"filters": filters}))
            .await?;
        Ok(())
    }

    /// The exception a stopped thread is paused on (`exceptionInfo`).
    ///
    /// # Errors
    /// DAP request error.
    pub async fn exception_info(&self, thread_id: i64) -> Result<ExceptionInfo, DapError> {
        self.request("exceptionInfo", json!({"threadId": thread_id}))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::ExceptionInfo;

    #[test]
    fn the_headline_is_the_full_type_then_the_message_as_vs_code_shows_it() {
        let netcoredbg: ExceptionInfo = serde_json::from_str(
            r#"{"exceptionId":"CLR/System.InvalidOperationException","breakMode":"always",
                "description":"Exception thrown: 'System.InvalidOperationException' in Debuggee.dll: 'total was 14'",
                "details":{"fullTypeName":"System.InvalidOperationException","message":"total was 14","typeName":"InvalidOperationException"}}"#,
        )
        .unwrap();
        assert_eq!(
            netcoredbg.headline(),
            "System.InvalidOperationException: total was 14"
        );
        let plain: ExceptionInfo =
            serde_json::from_str(r#"{"exceptionId":"CLR/System.Exception","description":"boom"}"#)
                .unwrap();
        assert_eq!(
            plain.headline(),
            "System.Exception: boom",
            "no details: the id without its category, then the description"
        );
        let bare: ExceptionInfo =
            serde_json::from_str(r#"{"exceptionId":"System.Exception","description":"  "}"#)
                .unwrap();
        assert_eq!(bare.headline(), "System.Exception");
    }
}
