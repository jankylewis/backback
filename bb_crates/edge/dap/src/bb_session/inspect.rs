//! Inspect stopped threads, frames, scopes, and values.

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
}
