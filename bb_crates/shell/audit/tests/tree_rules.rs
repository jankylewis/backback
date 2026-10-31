//! Folder-level rules on throwaway trees: HD cap, underscore rule per tree
//! kind, file caps, and C# `var` found through the walk.

use std::path::{Path, PathBuf};

use bb_audit::{audit, Rule, Violation};

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-audit-{tag}-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn file(&self, rel: &str, text: &str) {
        let path: PathBuf = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn rules(&self) -> Vec<(String, Rule)> {
        let found: Vec<Violation> = audit(&self.0).unwrap();
        found
            .into_iter()
            .map(|v: Violation| (v.path.display().to_string(), v.rule))
            .collect()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
    }
}

fn root(s: &Scratch) -> &Path {
    &s.0
}

#[test]
fn six_entries_break_the_hd_cap_and_dotfiles_do_not_count() {
    let s: Scratch = Scratch::new("hd");
    for name in ["a.md", "b.md", "c.md", "d.md", "e.md", ".hidden"] {
        s.file(name, "x\n");
    }
    assert!(s.rules().is_empty(), "five visible + one hidden is fine");
    s.file("f.md", "x\n");
    assert_eq!(s.rules(), vec![(String::new(), Rule::FolderEntries)]);
}

#[test]
fn csharp_tree_needs_exactly_one_underscore_entry() {
    let s: Scratch = Scratch::new("cs");
    s.file("App/App.csproj", "<Project />\n");
    s.file("Lib/Lib.csproj", "<Project />\n");
    assert_eq!(s.rules(), vec![(String::new(), Rule::Underscore)]);
    std::fs::rename(root(&s).join("App"), root(&s).join("_App")).unwrap();
    assert!(s.rules().is_empty());
    std::fs::rename(root(&s).join("Lib"), root(&s).join("_Lib")).unwrap();
    assert_eq!(s.rules(), vec![(String::new(), Rule::Underscore)]);
}

#[test]
fn rust_and_mixed_trees_forbid_underscores() {
    let s: Scratch = Scratch::new("rs");
    s.file("Cargo.toml", "[workspace]\n");
    s.file("_core/lib.rs", "fn f() {}\n");
    s.file("fixtures/_App/App.csproj", "<Project />\n");
    assert_eq!(s.rules(), vec![(String::new(), Rule::Underscore)]);
}

#[test]
fn file_caps_and_csharp_var_are_reported_with_paths() {
    let s: Scratch = Scratch::new("caps");
    s.file("notes.md", &"line\n".repeat(101));
    s.file(
        "_App/App.cs",
        "public class A\n{\n    void M()\n    {\n        var x = 1;\n    }\n}\n",
    );
    let rules: Vec<(String, Rule)> = s.rules();
    assert!(
        rules.contains(&("notes.md".to_owned(), Rule::FileLines)),
        "{rules:?}"
    );
    assert!(
        rules.contains(&("_App/App.cs".to_owned(), Rule::CsVar)),
        "{rules:?}"
    );
}
