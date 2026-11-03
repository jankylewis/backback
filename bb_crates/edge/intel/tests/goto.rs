//! Go to Implementation and Go to Type Definition (BBEOD22) and the callers
//! of a method (BBEOD24) against the real roslyn-language-server on the
//! harness.

#[cfg(test)]
mod common;

use std::path::PathBuf;

use bb_intel::{CallItem, DocPath, DocUri, IncomingCall, Location};
use bb_lsp::roslyn::{self, Restore};
use bb_lsp::LspClient;
use bb_text::LspPoint;
use common::{start, Harness, DEADLINE};
use lsp_types::{CallHierarchyServerCapability, InitializeResult};

const SPEC: &str = "_NUnitApex/GreeterSpec.cs";
const GREETER: &str = "Core/Greeter.cs";
/// `string actual = greeter.Hello(name);`, 0-based.
const CALL_LINE: u32 = 13;
/// `public string Hello(string name)`, 0-based.
const HELLO_LINE: u32 = 4;

/// The point of the first `needle` on 0-based `line` of `text`.
fn point(text: &str, line: u32, needle: &str) -> Option<LspPoint> {
    let row: &str = text.lines().nth(usize::try_from(line).ok()?)?;
    let column: u32 = u32::try_from(row.find(needle)?).ok()?;
    Some(LspPoint {
        line,
        character: column,
    })
}

/// Whether `location` starts on 0-based `line` of the harness file `file`.
fn in_file(location: &Location, file: &str, line: u32) -> bool {
    let named: bool = match &location.uri {
        DocUri::File(path) => path.as_path().ends_with(file),
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
        types.iter().any(|l: &Location| in_file(l, GREETER, 2)),
        "the type of `greeter` is the class on line 2: {types:?}"
    );
    let members: Vec<Location> = h
        .session
        .implementation(&spec, point(&text, CALL_LINE, "Hello").unwrap(), DEADLINE)
        .await
        .unwrap();
    assert!(
        members.iter().any(|l: &Location| in_file(l, GREETER, 4)),
        "`Hello` is implemented on line 4: {members:?}"
    );
    h.finish().await;
}

/// BBEOD24: the server bb runs, started and initialized as `Session::start`
/// does it, advertises `callHierarchyProvider`.
#[tokio::test]
async fn the_server_advertises_call_hierarchy() {
    let base: PathBuf = std::env::temp_dir().join(format!("bb-intel-caps-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let client: LspClient = roslyn::start(
        &roslyn::locate().unwrap(),
        &base.join("logs"),
        DEADLINE,
        Restore::Off,
        &base.join("scratch"),
    )
    .unwrap();
    let init: InitializeResult = roslyn::initialize(&client, &base).await.unwrap();
    client.shutdown().await.unwrap();
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&base);
    let provider: Option<CallHierarchyServerCapability> = init.capabilities.call_hierarchy_provider;
    eprintln!("callHierarchyProvider: {provider:?}");
    assert!(
        matches!(
            provider,
            Some(
                CallHierarchyServerCapability::Simple(true)
                    | CallHierarchyServerCapability::Options(_)
            )
        ),
        "{provider:?}"
    );
}

/// BBEOD24: `Greeter.Hello` has one caller, the spec's `Greets`, which calls
/// it once, on the call line.
#[tokio::test]
async fn the_one_caller_of_hello_is_the_spec_at_its_call() {
    let mut h: Harness = start("calls").await;
    let greeter: DocPath = h.doc(GREETER);
    h.session.open(&greeter).await.unwrap();
    let text: String = std::fs::read_to_string(greeter.as_path()).unwrap();
    let spec: String = std::fs::read_to_string(h.root.join(SPEC)).unwrap();
    let call: LspPoint = point(&spec, CALL_LINE, "Hello").unwrap();
    let hello: LspPoint = point(&text, HELLO_LINE, "Hello").unwrap();
    let items: Vec<CallItem> = h
        .session
        .prepare_call_hierarchy(&greeter, hello, DEADLINE)
        .await
        .unwrap();
    let mut calls: Vec<IncomingCall> = Vec::new();
    for item in &items {
        calls.extend(h.session.incoming_calls(item, DEADLINE).await.unwrap());
    }
    h.finish().await;
    eprintln!("prepared: {items:?}\nincoming: {calls:?}");
    assert_eq!(items.len(), 1, "{items:?}");
    assert!(items[0].name.contains("Hello"), "{items:?}");
    assert!(
        in_file(&items[0].location, GREETER, HELLO_LINE),
        "{items:?}"
    );
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(calls[0].from.name.contains("Greets"), "{calls:?}");
    assert_eq!(calls[0].sites.len(), 1, "{calls:?}");
    let site: &Location = &calls[0].sites[0];
    assert!(in_file(site, SPEC, CALL_LINE), "{site:?}");
    assert!(
        site.span.start.character <= call.character && site.span.end.character > call.character,
        "the site holds the `Hello` of the call at {call:?}: {site:?}"
    );
}
