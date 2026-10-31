//! Backbone marker rules: `_` in C# trees, `bb_` in Rust and neutral trees,
//! at most one per folder, one required with 2+ visible subfolders, and a
//! Cargo crate root (with everything under it) exempt from the requirement.

#[cfg(test)]
mod common;

use bb_audit::Rule;
use common::{only, Scratch};

const WORKSPACE: &str = "[workspace]\n";
const PACKAGE: &str = "[package]\nname = \"k\"\n";

#[test]
fn csharp_tree_needs_exactly_one_underscore_entry() {
    let s: Scratch = Scratch::new("m-cs");
    s.file("App/App.csproj", "<Project />\n");
    s.file("Lib/Lib.csproj", "<Project />\n");
    assert_eq!(s.rules(), only("", Rule::Marker));
    s.rename("App", "_App");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
    s.rename("Lib", "_Lib");
    assert_eq!(s.rules(), only("", Rule::Marker));
}

#[test]
fn rust_tree_needs_exactly_one_bb_entry() {
    let s: Scratch = Scratch::new("m-rs");
    s.file("Cargo.toml", WORKSPACE);
    s.file("core/lib.rs", "fn f() {}\n");
    s.file("edge/lib.rs", "fn g() {}\n");
    assert_eq!(s.rules(), only("", Rule::Marker));
    s.rename("core", "bb_core");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
    s.rename("edge", "bb_edge");
    assert_eq!(s.rules(), only("", Rule::Marker));
}

#[test]
fn each_tree_rejects_the_other_marker() {
    let s: Scratch = Scratch::new("m-wrong");
    s.file("bb_rs/Cargo.toml", WORKSPACE);
    s.file("bb_rs/_core/lib.rs", "fn f() {}\n");
    s.file("bb_rs/bb_edge/lib.rs", "fn g() {}\n");
    s.file("cs/bb_App/App.csproj", "<Project />\n");
    s.file("cs/_Lib/Lib.csproj", "<Project />\n");
    let mut want: Vec<(String, Rule)> = only("bb_rs", Rule::Marker);
    want.push(("cs".to_owned(), Rule::Marker));
    assert_eq!(s.rules(), want);
}

#[test]
fn csharp_tree_inside_a_rust_repo_keeps_underscore() {
    let s: Scratch = Scratch::new("m-mixed");
    s.file("bb_crates/k.rs", "fn f() {}\n");
    s.file("fixtures/_App/App.csproj", "<Project />\n");
    s.file("fixtures/Lib/Lib.csproj", "<Project />\n");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
}

#[test]
fn neutral_folder_follows_the_repo_dialect() {
    for (code, marker) in [
        ("bb_crates/k.rs", "bb_"),
        ("_App/App.csproj", "_"),
        ("bb_notes/n.txt", "bb_"),
    ] {
        let s: Scratch = Scratch::new("m-neutral");
        s.file(code, "x\n");
        s.file("docs/plan/a.md", "x\n");
        s.file("docs/rnd/b.md", "x\n");
        assert_eq!(s.rules(), only("docs", Rule::Marker), "{code}");
        s.rename("docs/plan", &format!("docs/{marker}plan"));
        assert!(s.rules().is_empty(), "{code}: {:?}", s.rules());
    }
}

#[test]
fn cargo_crate_root_and_its_folders_are_exempt() {
    let s: Scratch = Scratch::new("m-crate");
    s.file("Cargo.toml", PACKAGE);
    s.file("src/a/x.rs", "fn f() {}\n");
    s.file("src/b/y.rs", "fn g() {}\n");
    s.file("tests/t.rs", "fn t() {}\n");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
    s.rename("src/a", "src/_a");
    assert_eq!(s.rules(), only("src", Rule::Marker), "exempt, not lawless");
}
