//! Folder-level rules on throwaway trees: HD cap, agent files, file caps, and
//! C# `var` and vacuous test skips found through the walk.

#[cfg(test)]
mod common;

use bb_audit::Rule;
use common::{only, Scratch};

#[test]
fn six_entries_break_the_hd_cap_and_dotfiles_do_not_count() {
    let s: Scratch = Scratch::new("hd");
    for name in ["bb_a.md", "b.md", "c.md", "d.md", "e.md", ".hidden"] {
        s.file(name, "x\n");
    }
    assert!(s.rules().is_empty(), "five visible + one hidden is fine");
    s.file("f.md", "x\n");
    assert_eq!(s.rules(), only("", Rule::FolderEntries));
}

#[test]
fn agent_files_do_not_count_toward_the_hd_cap() {
    let s: Scratch = Scratch::new("agents");
    for name in [
        "bb_a.md",
        "b.md",
        "c.md",
        "d.md",
        "e.md",
        "AGENTS.md",
        "CLAUDE.md",
    ] {
        s.file(name, "x\n");
    }
    assert!(s.rules().is_empty(), "{:?}", s.rules());
    s.file("AGENT.md", "x\n");
    assert_eq!(s.rules(), only("", Rule::FolderEntries), "exact names only");
}

#[test]
fn file_caps_and_csharp_var_are_reported_with_paths() {
    let s: Scratch = Scratch::new("caps");
    s.file("notes.md", &"line\n".repeat(101));
    s.file(
        "_App/App.cs",
        "public class A\n{\n    void M()\n    {\n        var x = 1;\n    }\n}\n",
    );
    s.rename("_App", "App");
    let rules: Vec<(String, Rule)> = s.rules();
    assert!(
        rules.contains(&("notes.md".to_owned(), Rule::FileLines)),
        "{rules:?}"
    );
    assert!(
        rules.contains(&("App/App.cs".to_owned(), Rule::CsVar)),
        "{rules:?}"
    );
}

#[test]
fn vacuous_tool_skip_is_reported_in_test_files_only() {
    let s: Scratch = Scratch::new("skip");
    let skip: &str = "fn t() {\n    if bb_dap::locate().is_err() {\n        return;\n    }\n}\n";
    s.file("crate/tests/live.rs", skip);
    s.file("crate/src/lib.rs", skip);
    let rules: Vec<(String, Rule)> = s.rules();
    assert!(
        rules.contains(&("crate/tests/live.rs".to_owned(), Rule::VacuousSkip)),
        "{rules:?}"
    );
    assert!(
        !rules.contains(&("crate/src/lib.rs".to_owned(), Rule::VacuousSkip)),
        "{rules:?}"
    );
}

#[test]
fn agent_skill_trees_keep_their_tool_layout_but_not_their_line_cap() {
    let s: Scratch = Scratch::new("skills");
    s.file("bb_a.md", "x\n");
    for tree in [".claude/skills/cca", ".agents/skills/cda"] {
        for n in 1..=7 {
            s.file(&format!("{tree}-{n}/SKILL.md"), "x\n");
        }
    }
    assert!(s.rules().is_empty(), "{:?}", s.rules());
    s.file(".claude/skills/cca-1/SKILL.md", &"line\n".repeat(101));
    assert_eq!(
        s.rules(),
        only(".claude/skills/cca-1/SKILL.md", Rule::FileLines)
    );
    for n in 1..=6 {
        s.file(&format!("docs/skills/s{n}.md"), "x\n");
    }
    assert!(
        s.rules()
            .contains(&("docs/skills".to_owned(), Rule::FolderEntries)),
        "only the root agent trees are exempt"
    );
}
