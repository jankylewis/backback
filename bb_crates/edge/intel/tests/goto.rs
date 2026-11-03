//! Go to Implementation and Go to Type Definition against the real
//! roslyn-language-server on the harness (BBEOD22).

#[cfg(test)]
mod common;

use bb_intel::{DocPath, DocUri, Location};
use bb_text::LspPoint;
use common::{start, Harness, DEADLINE};

const SPEC: &str = "_NUnitApex/GreeterSpec.cs";
/// `string actual = greeter.Hello(name);`, 0-based.
const CALL_LINE: u32 = 13;

/// The point of the first `needle` on 0-based `line` of `text`.
fn point(text: &str, line: u32, needle: &str) -> Option<LspPoint> {
    let row: &str = text.lines().nth(usize::try_from(line).ok()?)?;
    let column: u32 = u32::try_from(row.find(needle)?).ok()?;
    Some(LspPoint {
        line,
        character: column,
    })
}

fn in_greeter(location: &Location, line: u32) -> bool {
    let named: bool = match &location.uri {
        DocUri::File(path) => path.as_path().ends_with("Core/Greeter.cs"),
        DocUri::Virtual(_) => false,
    };
    named && location.span.start.line == line
}

#[tokio::test]
async fn type_definition_and_implementation_land_on_the_greeter() {
    let mut h: Harness = start("goto").await;
    let spec: DocPath = h.doc(SPEC);
    h.session.open(&spec).await.unwrap();
    let text: String = std::fs::read_to_string(spec.as_path()).unwrap();
    let types: Vec<Location> = h
        .session
        .type_definition(&spec, point(&text, CALL_LINE, "greeter").unwrap(), DEADLINE)
        .await
        .unwrap();
    assert!(
        types.iter().any(|l: &Location| in_greeter(l, 2)),
        "the type of `greeter` is the class on line 2: {types:?}"
    );
    let members: Vec<Location> = h
        .session
        .implementation(&spec, point(&text, CALL_LINE, "Hello").unwrap(), DEADLINE)
        .await
        .unwrap();
    assert!(
        members.iter().any(|l: &Location| in_greeter(l, 4)),
        "`Hello` is implemented on line 4: {members:?}"
    );
    h.finish().await;
}
