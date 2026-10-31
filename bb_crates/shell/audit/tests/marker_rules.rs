//! Backbone marker rules: `_` in C# trees, `bb_` in Rust and neutral trees.
//! Every folder with 2+ visible entries (files and folders) carries exactly
//! one marker on a non-fixed entry; a folder of fixed names only needs none.

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
fn files_only_folder_without_a_marker_is_a_violation() {
    let s: Scratch = Scratch::new("m-files");
    s.file("k/client.rs", "fn f() {}\n");
    s.file("k/wire.rs", "fn g() {}\n");
    assert_eq!(s.rules(), only("k", Rule::Marker));
    s.rename("k/client.rs", "k/bb_client.rs");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
}

#[test]
fn folder_with_two_markers_is_a_violation() {
    let s: Scratch = Scratch::new("m-two");
    s.file("k/bb_client.rs", "fn f() {}\n");
    s.file("k/bb_wire.rs", "fn g() {}\n");
    assert_eq!(s.rules(), only("k", Rule::Marker));
}

#[test]
fn crate_folder_of_fixed_names_needs_no_marker() {
    let s: Scratch = Scratch::new("m-crate");
    s.file("k/Cargo.toml", PACKAGE);
    s.file("k/README.md", "x\n");
    s.file("k/src/lib.rs", "fn f() {}\n");
    s.file("k/tests/t.rs", "fn t() {}\n");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
}

#[test]
fn fixed_names_never_count_toward_the_marker() {
    let s: Scratch = Scratch::new("m-fixed");
    s.file("k/Cargo.toml", PACKAGE);
    s.file("k/src/lib.rs", "fn f() {}\n");
    s.file("k/src/position.rs", "fn g() {}\n");
    assert_eq!(s.rules(), only("k/src", Rule::Marker));
    s.rename("k/src/position.rs", "k/src/bb_position.rs");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
}

#[test]
fn csharp_project_folder_needs_underscore_on_one_entry() {
    let s: Scratch = Scratch::new("m-csproj");
    s.file("App/App.csproj", "<Project />\n");
    s.file("App/Program.cs", "class P {}\n");
    assert_eq!(s.rules(), only("App", Rule::Marker));
    s.rename("App/App.csproj", "App/_App.csproj");
    assert!(s.rules().is_empty(), "{:?}", s.rules());
}
