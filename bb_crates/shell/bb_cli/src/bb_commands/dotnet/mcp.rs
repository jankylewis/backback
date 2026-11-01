//! MCP project discovery and fixture-safe stdio commands.

use std::path::Path;

use bb_dotnet::{Capability, ProjectInfo, SolutionPath};
use bb_mcp::{McpError, Session, Tool};
use serde_json::Value;

use crate::cli::McpCommand;

use super::super::CommandError;
use super::scan;

/// MCP CLI errors.
#[derive(Debug, thiserror::Error)]
pub enum McpCommandError {
    /// Discovery failed.
    #[error(transparent)]
    Discovery(#[from] Box<CommandError>),
    /// Stdio or protocol failed.
    #[error(transparent)]
    Session(#[from] McpError),
    /// Tool arguments were not JSON.
    #[error("invalid JSON arguments: {0}")]
    Arguments(#[from] serde_json::Error),
    /// A project is not a runnable MCP server.
    #[error("not a runnable MCP server: {0}")]
    NotServer(String),
    /// The server returned a tool-level error.
    #[error("tool {0} returned isError")]
    Tool(String),
}

/// Execute a parsed MCP command.
///
/// # Errors
/// Discovery, protocol, and process failures.
pub async fn run(command: McpCommand) -> Result<(), McpCommandError> {
    match command {
        McpCommand::List(path) => list(&path),
        McpCommand::Tools(path) => {
            let mut session: Session = start(&path).await?;
            let tools: Vec<Tool> = session.tools().await?;
            for tool in tools {
                println!("{}  {}", tool.name, tool.description.unwrap_or_default());
            }
            session.stop().await?;
            Ok(())
        }
        McpCommand::Call {
            project,
            tool,
            arguments,
        } => {
            let value: Value = serde_json::from_str(&arguments)?;
            let mut session: Session = start(&project).await?;
            let result: bb_mcp::CallResult = session.call(&tool, value).await?;
            println!("{}", serde_json::to_string(&result.content)?);
            session.stop().await?;
            if result.is_error {
                return Err(McpCommandError::Tool(tool));
            }
            Ok(())
        }
        McpCommand::Log(path) => log(&path).await,
    }
}

async fn log(path: &Path) -> Result<(), McpCommandError> {
    let mut session: Session = start(path).await?;
    let logs: &mut tokio::sync::broadcast::Receiver<String> = session.logs();
    loop {
        tokio::select! {
            line = logs.recv() => match line {
                Ok(text) => eprintln!("{text}"),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|source: std::io::Error| McpError::Io {
                    operation: "wait for Ctrl-C",
                    source,
                })?;
                break;
            }
        }
    }
    session.stop().await?;
    Ok(())
}

fn list(solution: &Path) -> Result<(), McpCommandError> {
    let sln: SolutionPath = SolutionPath::new(solution.to_path_buf())
        .map_err(|error: bb_dotnet::SolutionError| Box::new(CommandError::Solution(error)))?;
    let infos: Vec<ProjectInfo> = scan::load(&sln).map_err(Box::new)?;
    for info in infos
        .iter()
        .filter(|info: &&ProjectInfo| info.has(Capability::Mcp))
    {
        let run: String = if info.is_exe {
            format!("dotnet run --project {}", info.path.as_path().display())
        } else {
            "not executable".to_owned()
        };
        println!("{}  {run}", info.path.as_path().display());
    }
    Ok(())
}

async fn start(project: &Path) -> Result<Session, McpCommandError> {
    let path: bb_dotnet::ProjectPath = bb_dotnet::ProjectPath::from_path(project.to_path_buf());
    let info: ProjectInfo = ProjectInfo::load(&path)
        .map_err(|error: bb_dotnet::ProjectError| Box::new(CommandError::Project(error)))?;
    if !info.has(Capability::Mcp) || !info.is_exe {
        return Err(McpCommandError::NotServer(project.display().to_string()));
    }
    Ok(Session::start(project).await?)
}
