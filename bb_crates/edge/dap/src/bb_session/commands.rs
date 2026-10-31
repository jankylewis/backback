//! Typed commands over the bounded DAP request path.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{Capabilities, DapError, SourceBreakpoint};

use super::lifecycle::{state::Action, Target};
use super::DapSession;

#[derive(Deserialize)]
struct TargetsBody {
    targets: Vec<GotoTarget>,
}

/// One valid destination for Set Next Statement.
#[derive(Debug, Deserialize)]
pub struct GotoTarget {
    /// Adapter target ID.
    pub id: i64,
    /// Display label.
    pub label: String,
}

impl DapSession {
    /// Initialize the adapter and retain its capabilities.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn initialize(&self) -> Result<Capabilities, DapError> {
        let next: super::SessionState = self.require_action(Action::Initialize)?;
        let caps: Capabilities = self
            .request(
                "initialize",
                json!({
                    "clientID": "backback", "adapterID": "coreclr", "pathFormat": "path",
                    "linesStartAt1": true, "columnsStartAt1": true,
                    "supportsVariableType": true, "supportsRunInTerminalRequest": false
                }),
            )
            .await?;
        *self.capabilities.lock().map_err(|_: _| DapError::Closed)? = Some(caps.clone());
        self.commit_state(next)?;
        Ok(caps)
    }

    /// Start a managed program.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn launch(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        just_my_code: bool,
    ) -> Result<(), DapError> {
        let next: super::SessionState = self.require_action(Action::Launch)?;
        let _: Value = self
            .request(
                "launch",
                json!({
                    "name": "backback", "type": "coreclr", "request": "launch",
                    "program": program, "args": args, "cwd": cwd, "env": env,
                    "justMyCode": just_my_code, "stopAtEntry": false
                }),
            )
            .await?;
        self.set_target(Target::Launched)?;
        self.commit_state(next)?;
        Ok(())
    }

    /// Attach to a running managed process.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn attach(&self, process_id: u32) -> Result<(), DapError> {
        let next: super::SessionState = self.require_action(Action::Attach)?;
        let _: Value = self
            .request(
                "attach",
                json!({
                    "name": "backback", "type": "coreclr", "request": "attach",
                    "processId": process_id.to_string(), "justMyCode": false
                }),
            )
            .await?;
        self.set_target(Target::Attached(process_id))?;
        self.commit_state(next)?;
        Ok(())
    }

    /// Replace the breakpoints for a source file.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn set_breakpoints(
        &self,
        source: &Path,
        points: &[SourceBreakpoint],
    ) -> Result<Value, DapError> {
        self.request(
            "setBreakpoints",
            json!({
                "source": {"path": source}, "breakpoints": points,
                "sourceModified": false
            }),
        )
        .await
    }

    /// Finish breakpoint configuration.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn configuration_done(&self) -> Result<(), DapError> {
        let next: super::SessionState = self.require_action(Action::ConfigurationDone)?;
        let _: Value = self.request("configurationDone", json!({})).await?;
        self.commit_state(next)?;
        Ok(())
    }

    /// Resume a stopped thread.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn continue_thread(&self, thread_id: i64) -> Result<(), DapError> {
        let _: Value = self
            .request("continue", json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    /// Step over, into, or out of a frame.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn step(&self, command: Step, thread_id: i64) -> Result<(), DapError> {
        let _: Value = self
            .request(command.name(), json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    /// Pause one running thread.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn pause(&self, thread_id: i64) -> Result<(), DapError> {
        let _: Value = self
            .request("pause", json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    /// Ask the adapter to terminate the target.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn terminate(&self) -> Result<(), DapError> {
        let _: Value = self.request("terminate", json!({})).await?;
        Ok(())
    }

    /// Disconnect, optionally ending the target.
    ///
    /// # Errors
    /// DAP request error.
    pub async fn disconnect(&self, terminate_debuggee: bool) -> Result<(), DapError> {
        let _: Value = self
            .request(
                "disconnect",
                json!({"terminateDebuggee": terminate_debuggee}),
            )
            .await?;
        Ok(())
    }

    /// Return valid Set Next Statement targets when the adapter supports them.
    ///
    /// # Errors
    /// Unsupported capability or DAP request error.
    pub async fn goto_targets(
        &self,
        source: &Path,
        line: u32,
    ) -> Result<Vec<GotoTarget>, DapError> {
        if !self.can_set_next_statement() {
            return Err(DapError::Unsupported("Set Next Statement"));
        }
        Ok(self
            .request::<TargetsBody>(
                "gotoTargets",
                json!({"source": {"path": source}, "line": line}),
            )
            .await?
            .targets)
    }

    /// Set the next instruction to a target advertised by `gotoTargets`.
    ///
    /// # Errors
    /// Unsupported capability or DAP request error.
    pub async fn goto(&self, thread_id: i64, target_id: i64) -> Result<(), DapError> {
        if !self.can_set_next_statement() {
            return Err(DapError::Unsupported("Set Next Statement"));
        }
        let _: Value = self
            .request(
                "goto",
                json!({"threadId": thread_id, "targetId": target_id}),
            )
            .await?;
        Ok(())
    }
}

/// Supported stepping operations.
#[derive(Debug, Clone, Copy)]
pub enum Step {
    /// Step over.
    Next,
    /// Step into.
    In,
    /// Step out.
    Out,
}

impl Step {
    fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::In => "stepIn",
            Self::Out => "stepOut",
        }
    }
}
