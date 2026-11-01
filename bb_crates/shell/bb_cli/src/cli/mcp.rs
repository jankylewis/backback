//! MCP command arguments.

use super::CliError;
use std::path::PathBuf;

/// A parsed MCP subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpCommand {
    /// List MCP server projects in a solution.
    List(PathBuf),
    /// List server tools.
    Tools(PathBuf),
    /// Call a tool with JSON arguments.
    Call {
        project: PathBuf,
        tool: String,
        arguments: String,
    },
    /// Follow server stderr.
    Log(PathBuf),
}

pub(super) fn parse(rest: &[String]) -> Result<McpCommand, CliError> {
    match rest {
        [word, path] if word == "list" => Ok(McpCommand::List(PathBuf::from(path))),
        [word, path] if word == "tools" => Ok(McpCommand::Tools(PathBuf::from(path))),
        [word, path] if word == "log" => Ok(McpCommand::Log(PathBuf::from(path))),
        [word, path, tool, arguments] if word == "call" => Ok(McpCommand::Call {
            project: PathBuf::from(path),
            tool: tool.clone(),
            arguments: arguments.clone(),
        }),
        [] => Err(CliError::NeedsPath("mcp")),
        [word] if ["list", "tools", "call", "log"].contains(&word.as_str()) => {
            Err(CliError::NeedsPath("mcp"))
        }
        [word, ..] if ["list", "tools", "call", "log"].contains(&word.as_str()) => {
            Err(CliError::Extra(rest.last().cloned().unwrap_or_default()))
        }
        [word, ..] => Err(CliError::McpCommand(word.clone())),
    }
}
