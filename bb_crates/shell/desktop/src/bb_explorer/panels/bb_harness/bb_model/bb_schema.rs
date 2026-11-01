//! JSON-schema to form fields, and form fields back to JSON arguments.

use serde_json::{Map, Number, Value};

use super::{FieldKind, FormField, HarnessError};

pub(crate) fn fields(schema: &Value) -> Vec<FormField> {
    let mut output: Vec<FormField> = Vec::new();
    collect(schema, &[], &[], &mut output);
    output
}

fn collect(schema: &Value, prefix: &[String], parents: &[bool], output: &mut Vec<FormField>) {
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
    for (name, raw) in properties {
        let (property, kind, nullable): (&Value, Option<&str>, bool) = effective(raw);
        let mut path: Vec<String> = prefix.to_vec();
        path.push(name.clone());
        let is_required: bool = required.contains(&name.as_str());
        if kind == Some("object") && has_properties(property) {
            let mut chain: Vec<bool> = parents.to_vec();
            chain.push(is_required);
            collect(property, &path, &chain, output);
            continue;
        }
        output.push(FormField {
            path,
            kind: leaf_kind(property, kind),
            value: String::new(),
            required: is_required,
            parents: parents.to_vec(),
            nullable,
        });
    }
}

fn has_properties(schema: &Value) -> bool {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .is_some_and(|map: &Map<String, Value>| !map.is_empty())
}

fn is_null_schema(schema: &Value) -> bool {
    schema.get("type").and_then(Value::as_str) == Some("null")
}

/// The schema that carries the type, its type name, and whether `null` is allowed.
fn effective(schema: &Value) -> (&Value, Option<&str>, bool) {
    let variants: Option<&Vec<Value>> = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array);
    if let Some(variants) = variants {
        let nullable: bool = variants.iter().any(is_null_schema);
        if let Some(inner) = variants
            .iter()
            .find(|variant: &&Value| !is_null_schema(variant))
        {
            let (found, kind, inner_nullable): (&Value, Option<&str>, bool) = effective(inner);
            return (found, kind, nullable || inner_nullable);
        }
    }
    match schema.get("type") {
        Some(Value::String(kind)) => (schema, Some(kind.as_str()), false),
        Some(Value::Array(kinds)) => (
            schema,
            kinds
                .iter()
                .filter_map(Value::as_str)
                .find(|kind: &&str| *kind != "null"),
            kinds.iter().any(|kind: &Value| kind == "null"),
        ),
        Some(Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_)) | None => {
            (schema, has_properties(schema).then_some("object"), false)
        }
    }
}

fn leaf_kind(schema: &Value, kind: Option<&str>) -> FieldKind {
    if let Some(options) = schema.get("enum").and_then(Value::as_array) {
        return FieldKind::Enum(
            options
                .iter()
                .filter(|o: &&Value| !o.is_null())
                .cloned()
                .collect(),
        );
    }
    match kind {
        Some("integer") => FieldKind::Integer,
        Some("number") => FieldKind::Number,
        Some("boolean") => FieldKind::Boolean,
        Some("object") => FieldKind::Json,
        Some("array") => {
            let items: Value = schema.get("items").cloned().unwrap_or(Value::Null);
            let (item, item_kind, _nullable_items_take_null_from_json): (
                &Value,
                Option<&str>,
                bool,
            ) = effective(&items);
            let item_kind: FieldKind = match item_kind {
                Some("array") => FieldKind::Json,
                other => leaf_kind(item, other),
            };
            FieldKind::Array(Box::new(item_kind))
        }
        _ => FieldKind::String,
    }
}

pub(crate) fn enum_label(option: &Value) -> String {
    option
        .as_str()
        .map_or_else(|| option.to_string(), str::to_owned)
}

pub(crate) fn arguments(fields: &[FormField]) -> Result<Value, HarnessError> {
    let mut root: Map<String, Value> = Map::new();
    for field in fields {
        let label: String = field.path.join(".");
        if field.value.is_empty() {
            if effectively_required(field, fields) {
                return Err(HarnessError::Field(label, "required".into()));
            }
            continue;
        }
        let value: Value = if field.nullable && field.value.trim() == "null" {
            Value::Null
        } else {
            convert(&field.kind, &field.value)
                .map_err(|reason: String| HarnessError::Field(label, reason))?
        };
        insert_nested(&mut root, &field.path, value);
    }
    Ok(Value::Object(root))
}

/// Required in its object, and every enclosing object is required or in use.
fn effectively_required(field: &FormField, fields: &[FormField]) -> bool {
    field.required
        && field
            .parents
            .iter()
            .enumerate()
            .all(|(depth, required): (usize, &bool)| {
                *required || in_use(fields, field.path.get(..=depth))
            })
}

fn in_use(fields: &[FormField], prefix: Option<&[String]>) -> bool {
    prefix.is_some_and(|prefix: &[String]| {
        fields
            .iter()
            .any(|other: &FormField| other.path.starts_with(prefix) && !other.value.is_empty())
    })
}

fn convert(kind: &FieldKind, text: &str) -> Result<Value, String> {
    match kind {
        FieldKind::String => Ok(Value::String(text.to_owned())),
        FieldKind::Integer => text
            .trim()
            .parse::<i64>()
            .map(Value::from)
            .map_err(|error: std::num::ParseIntError| format!("not an integer ({error})")),
        FieldKind::Number => text
            .trim()
            .parse::<Number>()
            .map(Value::Number)
            .map_err(|error: serde_json::Error| format!("not a number ({error})")),
        FieldKind::Boolean => text
            .trim()
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|error: std::str::ParseBoolError| error.to_string()),
        FieldKind::Enum(options) => options
            .iter()
            .find(|option: &&Value| enum_label(option) == text)
            .cloned()
            .ok_or_else(|| format!("outside enum {}", Value::Array(options.clone()))),
        FieldKind::Array(item) => array(item, text),
        FieldKind::Json => serde_json::from_str::<Value>(text)
            .map_err(|error: serde_json::Error| format!("not JSON ({error})")),
    }
}

fn array(item: &FieldKind, text: &str) -> Result<Value, String> {
    let trimmed: &str = text.trim();
    let items: Vec<Value> = if trimmed.starts_with('[') {
        let parsed: Vec<Value> = serde_json::from_str(trimmed)
            .map_err(|error: serde_json::Error| format!("not a JSON array ({error})"))?;
        parsed
            .into_iter()
            .map(|value: Value| typed_item(item, value))
            .collect::<Result<Vec<Value>, String>>()?
    } else {
        trimmed
            .split(',')
            .map(|part: &str| typed_item(item, convert(item, part.trim())?))
            .collect::<Result<Vec<Value>, String>>()?
    };
    Ok(Value::Array(items))
}

fn typed_item(item: &FieldKind, value: Value) -> Result<Value, String> {
    let fits: bool = match item {
        FieldKind::String => value.is_string(),
        FieldKind::Integer => value.is_i64() || value.is_u64(),
        FieldKind::Number => value.is_number(),
        FieldKind::Boolean => value.is_boolean(),
        FieldKind::Enum(options) => options.contains(&value),
        FieldKind::Array(_) => value.is_array(),
        FieldKind::Json => value.is_object(),
    };
    if fits {
        Ok(value)
    } else {
        Err(format!("item {value} does not match the item type"))
    }
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
