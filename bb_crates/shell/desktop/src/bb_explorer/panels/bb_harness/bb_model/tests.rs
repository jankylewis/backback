use super::*;
use serde_json::json;

fn rich_schema() -> Value {
    json!({"type":"object","required":["name","inner"],"properties":{
        "name":{"type":"string"},
        "inner":{"type":"object","required":["level"],"properties":{
            "level":{"type":"integer"},
            "deep":{"type":"object","properties":{"flag":{"type":"boolean"}}}}},
        "opt":{"type":"object","required":["must"],"properties":{"must":{"type":"string"}}},
        "opt2":{"type":"object","required":["must"],"properties":{
            "must":{"type":"string"},"extra":{"type":"string"}}},
        "tags":{"type":"array","items":{"type":"string"}},
        "counts":{"type":"array","items":{"type":"integer"}},
        "points":{"type":"array","items":{"type":"object","properties":{"x":{"type":"number"}}}},
        "color":{"type":"string","enum":["red","green"]},
        "size":{"type":"integer","enum":[1,2,3]},
        "ratio":{"type":"number","enum":[0.5,1.5]},
        "maybe":{"type":["object","null"],"properties":{"z":{"type":"string"}}},
        "note":{"type":["string","null"]},
        "when":{"anyOf":[{"type":"integer"},{"type":"null"}]}}})
}

fn call_with(values: &[(&str, &str)]) -> Result<Value, HarnessError> {
    let mut form: Vec<FormField> = fields(&rich_schema());
    for field in &mut form {
        let label: String = field.path.join(".");
        if let Some((_, value)) = values
            .iter()
            .find(|(path, _): &&(&str, &str)| *path == label)
        {
            field.value = (*value).to_owned();
        }
    }
    arguments(&form)
}

fn rejected(values: &[(&str, &str)]) -> String {
    call_with(values).unwrap_err().to_string()
}

const MINIMAL: [(&str, &str); 2] = [("name", "Ada"), ("inner.level", "3")];

fn minimal_and(extra: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    MINIMAL.iter().chain(extra.iter()).copied().collect()
}

#[test]
fn optional_object_with_required_child_is_omitted_until_one_child_is_filled() {
    assert_eq!(
        call_with(&MINIMAL).unwrap(),
        json!({"name":"Ada","inner":{"level":3}})
    );
    assert!(rejected(&minimal_and(&[("opt2.extra", "x")])).contains("opt2.must: required"));
    assert_eq!(
        call_with(&minimal_and(&[("opt2.extra", "x"), ("opt2.must", "y")])).unwrap()["opt2"],
        json!({"must":"y","extra":"x"})
    );
    assert!(rejected(&[("name", "Ada")]).contains("inner.level: required"));
}

#[test]
fn integer_and_number_enums_send_json_numbers() {
    let sent: Value = call_with(&minimal_and(&[
        ("size", "2"),
        ("ratio", "0.5"),
        ("color", "green"),
    ]))
    .unwrap();
    assert_eq!(sent["size"], json!(2));
    assert_eq!(sent["ratio"], json!(0.5));
    assert_eq!(sent["color"], json!("green"));
    assert!(rejected(&minimal_and(&[("size", "4")])).contains("size"));
}

#[test]
fn arrays_accept_comma_lists_and_json_arrays_with_typed_items() {
    let sent: Value = call_with(&minimal_and(&[
        ("tags", "a, b"),
        ("counts", "[1, 2]"),
        ("points", r#"[{"x":1.5}]"#),
    ]))
    .unwrap();
    assert_eq!(sent["tags"], json!(["a", "b"]));
    assert_eq!(sent["counts"], json!([1, 2]));
    assert_eq!(sent["points"], json!([{"x":1.5}]));
    assert!(rejected(&minimal_and(&[("points", "1, 2")])).contains("points"));
    assert!(rejected(&minimal_and(&[("counts", "1, x")])).contains("counts"));
}

#[test]
fn nullable_unions_accept_null_and_their_value_type() {
    let sent: Value = call_with(&minimal_and(&[
        ("maybe.z", "q"),
        ("note", "null"),
        ("when", "4"),
    ]))
    .unwrap();
    assert_eq!(sent["maybe"], json!({"z":"q"}));
    assert_eq!(sent["note"], Value::Null);
    assert_eq!(sent["when"], json!(4));
    assert_eq!(
        call_with(&minimal_and(&[("when", "null")])).unwrap()["when"],
        Value::Null
    );
}

#[test]
fn integer_field_rejects_a_fraction() {
    assert!(rejected(&[("name", "Ada"), ("inner.level", "1.5")]).contains("inner.level"));
}

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
