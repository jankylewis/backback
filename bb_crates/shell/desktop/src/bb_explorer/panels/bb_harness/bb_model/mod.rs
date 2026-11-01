use std::path::{Path, PathBuf};

use bb_dotnet::{Capability, ProjectInfo, Solution, SolutionPath};
use bb_mcp::Tool;
use serde_json::Value;

#[path = "bb_schema.rs"]
mod schema;
mod text;

pub(crate) use schema::{arguments, enum_label, fields};
pub(crate) use text::RowText;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ServerStatus {
    Stopped,
    Starting,
    Stopping,
    Running,
    Crashed(String),
}

#[derive(Clone, Debug)]
pub(crate) struct Server {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) status: ServerStatus,
    pub(crate) tools: Vec<ToolView>,
    pub(crate) logs: Vec<String>,
    pub(crate) result: Option<RowText>,
    pub(crate) error: Option<String>,
    pub(crate) generation: u64,
    pub(crate) pid: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct ToolView {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) fields: Vec<FormField>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FieldKind {
    String,
    Integer,
    Number,
    Boolean,
    Enum(Vec<Value>),
    Array(Box<FieldKind>),
    Json,
}

#[derive(Clone, Debug)]
pub(crate) struct FormField {
    pub(crate) path: Vec<String>,
    pub(crate) kind: FieldKind,
    pub(crate) value: String,
    /// Required within its own object.
    pub(crate) required: bool,
    /// For each enclosing object, whether its parent requires it.
    pub(crate) parents: Vec<bool>,
    /// The schema allows `null`; the text `null` sends it.
    pub(crate) nullable: bool,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HarnessError {
    #[error(transparent)]
    Solution(#[from] bb_dotnet::SolutionError),
    #[error(transparent)]
    Project(#[from] bb_dotnet::ProjectError),
    #[error("invalid field {0}: {1}")]
    Field(String, String),
}

pub(crate) fn discover(root: &Path) -> Result<Vec<Server>, HarnessError> {
    let mut servers: Vec<Server> = Vec::new();
    let solutions: Vec<PathBuf> = if root.is_file() {
        vec![root.to_path_buf()]
    } else {
        std::fs::read_dir(root)
            .map_err(|source: std::io::Error| bb_dotnet::SolutionError::Read {
                path: root.to_path_buf(),
                source,
            })?
            .filter_map(|entry: Result<std::fs::DirEntry, std::io::Error>| entry.ok())
            .map(|entry: std::fs::DirEntry| entry.path())
            .filter(|path: &PathBuf| {
                matches!(
                    path.extension().and_then(|e: &std::ffi::OsStr| e.to_str()),
                    Some("sln" | "slnx")
                )
            })
            .collect()
    };
    for path in solutions {
        let solution: Solution = Solution::load(&SolutionPath::new(path)?)?;
        for project in &solution.projects {
            let info: ProjectInfo = ProjectInfo::load(project)?;
            if info.has(Capability::Mcp) && info.is_exe {
                servers.push(Server {
                    name: info.name,
                    path: project.as_path().to_path_buf(),
                    status: ServerStatus::Stopped,
                    tools: Vec::new(),
                    logs: Vec::new(),
                    result: None,
                    error: None,
                    generation: 0,
                    pid: None,
                });
            }
        }
    }
    Ok(servers)
}

pub(crate) fn tool_views(tools: Vec<Tool>) -> Vec<ToolView> {
    tools
        .into_iter()
        .map(|tool: Tool| ToolView {
            name: tool.name,
            description: tool.description.unwrap_or_default(),
            fields: fields(&tool.input_schema),
        })
        .collect()
}

#[cfg(test)]
mod tests;
