use std::path::{Path, PathBuf};

use bb_dotnet::{Capability, ProjectInfo, Solution, SolutionPath};
use bb_mcp::Tool;
use serde_json::{Map, Number, Value};

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
    pub(crate) result: Option<String>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FieldKind {
    String,
    Number,
    Boolean,
    Enum(Vec<String>),
}

#[derive(Clone, Debug)]
pub(crate) struct FormField {
    pub(crate) path: Vec<String>,
    pub(crate) kind: FieldKind,
    pub(crate) value: String,
    pub(crate) required: bool,
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

pub(crate) fn fields(schema: &Value) -> Vec<FormField> {
    let mut output: Vec<FormField> = Vec::new();
    collect_fields(schema, &[], &mut output);
    output
}

fn collect_fields(schema: &Value, prefix: &[String], output: &mut Vec<FormField>) {
    let Some(properties): Option<&Map<String, Value>> =
        schema.get("properties").and_then(Value::as_object)
    else {
        return;
    };
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |values: &Vec<Value>| {
            values.iter().filter_map(Value::as_str).collect()
        });
    for (name, value) in properties {
        let mut path: Vec<String> = prefix.to_vec();
        path.push(name.clone());
        if value.get("type").and_then(Value::as_str) == Some("object") {
            collect_fields(value, &path, output);
            continue;
        }
        let kind: FieldKind = if let Some(options) = value.get("enum").and_then(Value::as_array) {
            FieldKind::Enum(
                options
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            )
        } else {
            match value.get("type").and_then(Value::as_str) {
                Some("number" | "integer") => FieldKind::Number,
                Some("boolean") => FieldKind::Boolean,
                _ => FieldKind::String,
            }
        };
        output.push(FormField {
            path,
            kind,
            value: String::new(),
            required: required.contains(&name.as_str()),
        });
    }
}

pub(crate) fn arguments(fields: &[FormField]) -> Result<Value, HarnessError> {
    let mut root: Map<String, Value> = Map::new();
    for field in fields {
        let label: String = field.path.join(".");
        if field.value.is_empty() {
            if field.required {
                return Err(HarnessError::Field(label, "required".into()));
            }
            continue;
        }
        let value: Value = match &field.kind {
            FieldKind::String => Value::String(field.value.clone()),
            FieldKind::Number => field.value.parse::<Number>().map(Value::Number).map_err(
                |error: serde_json::Error| HarnessError::Field(label.clone(), error.to_string()),
            )?,
            FieldKind::Boolean => field.value.parse::<bool>().map(Value::Bool).map_err(
                |error: std::str::ParseBoolError| {
                    HarnessError::Field(label.clone(), error.to_string())
                },
            )?,
            FieldKind::Enum(options) => {
                if !options.contains(&field.value) {
                    return Err(HarnessError::Field(label, "outside enum".into()));
                }
                Value::String(field.value.clone())
            }
        };
        insert_nested(&mut root, &field.path, value);
    }
    Ok(Value::Object(root))
}

fn insert_nested(root: &mut Map<String, Value>, path: &[String], value: Value) {
    let Some((head, tail)): Option<(&String, &[String])> = path.split_first() else {
        return;
    };
    if tail.is_empty() {
        root.insert(head.clone(), value);
        return;
    }
    let child: &mut Value = root
        .entry(head.clone())
        .or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(map) = child {
        insert_nested(map, tail, value);
    }
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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_forms_nested_basic_values_and_rejects_bad_input() {
        let schema: Value = json!({"type":"object","required":["name"],"properties":{
            "name":{"type":"string"},"count":{"type":"integer"},
            "enabled":{"type":"boolean"},"mode":{"enum":["a","b"]},
            "settings":{"type":"object","properties":{"tag":{"type":"string"}}}}});
        let mut form: Vec<FormField> = fields(&schema);
        assert_eq!(form.len(), 5);
        assert!(arguments(&form).is_err());
        for field in &mut form {
            field.value = match field.path.join(".").as_str() {
                "name" => "Ada",
                "count" => "3",
                "enabled" => "true",
                "mode" => "b",
                "settings.tag" => "x",
                _ => "",
            }
            .into();
        }
        assert_eq!(
            arguments(&form).unwrap(),
            json!({"name":"Ada","count":3,"enabled":true,"mode":"b","settings":{"tag":"x"}})
        );
        form.iter_mut()
            .find(|field: &&mut FormField| field.path == ["count"])
            .unwrap()
            .value = "oops".into();
        assert!(matches!(arguments(&form), Err(HarnessError::Field(_, _))));
    }
}
