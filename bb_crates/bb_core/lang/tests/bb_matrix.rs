//! BBEOD26: every non-empty tick set on every fixture folder: which tools
//! start, which files get colours, whether Debug shows. The expected values
//! come from the fixture table, never from `detect`.

use std::path::PathBuf;

use bb_lang::fixtures::{Fixture, FIXTURES};
use bb_lang::{detect, FileMode, Lang, LangSet, Langs, Tool, DETECT_BUDGET};

const SERVES: [(Tool, Lang); 7] = [
    (Tool::Roslyn, Lang::CSharp),
    (Tool::Netcoredbg, Lang::CSharp),
    (Tool::TestDiscovery, Lang::CSharp),
    (Tool::RustAnalyzer, Lang::Rust),
    (Tool::Cargo, Lang::Rust),
    (Tool::Pyright, Lang::Python),
    (Tool::Pytest, Lang::Python),
];

fn expected_tools(active: LangSet) -> Vec<Tool> {
    let mut tools: Vec<Tool> = Vec::new();
    if active.has(Lang::CSharp) {
        tools.push(Tool::Roslyn);
    }
    if active.has(Lang::Rust) {
        tools.push(Tool::RustAnalyzer);
        tools.push(Tool::Cargo);
    }
    if active.has(Lang::Python) {
        tools.push(Tool::Pyright);
    }
    tools
}

fn expected_mode(file_lang: Lang, active: LangSet) -> FileMode {
    if active.has(file_lang) {
        FileMode::Colours(file_lang)
    } else {
        FileMode::Plain
    }
}

fn check(fixture: &Fixture, root: &std::path::Path, ticked: LangSet) -> Vec<String> {
    let holds: LangSet = LangSet::of(fixture.langs);
    let active: LangSet = ticked.and(holds);
    let langs: Langs = Langs::new(ticked, detect(root, DETECT_BUDGET).found);
    let mut wrong: Vec<String> = Vec::new();
    let case: String = format!("{} ticked [{}]", fixture.name, ticked.names());
    if langs.active() != active {
        wrong.push(format!("{case}: active {:?}", langs.active()));
    }
    if langs.tools_at_open() != expected_tools(active) {
        wrong.push(format!("{case}: tools {:?}", langs.tools_at_open()));
    }
    for (tool, lang) in SERVES {
        if langs.allows(tool) != active.has(lang) {
            wrong.push(format!("{case}: {tool:?} allowed {}", langs.allows(tool)));
        }
    }
    if langs.debug_visible() != active.has(Lang::CSharp) {
        wrong.push(format!("{case}: debug {}", langs.debug_visible()));
    }
    for (file, lang) in fixture.sources() {
        let path: PathBuf = root.join(file);
        if langs.file_mode(&path) != expected_mode(lang, active) {
            wrong.push(format!("{case}: {file} {:?}", langs.file_mode(&path)));
        }
        let debug: bool = active.has(Lang::CSharp) && lang == Lang::CSharp;
        if langs.can_debug_from(&path) != debug {
            wrong.push(format!("{case}: debug from {file}"));
        }
    }
    let absent: LangSet = ticked.but_not(holds);
    for lang in Lang::ALL {
        let note: String = format!("No {} project here", lang.name());
        if langs.status().contains(&note) != absent.has(lang) {
            wrong.push(format!("{case}: status {:?}", langs.status()));
        }
    }
    wrong
}

#[test]
fn every_tick_set_on_every_fixture_starts_and_colours_only_the_active_languages() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    let mut passed: usize = 0;
    let mut wrong: Vec<String> = Vec::new();
    for fixture in &FIXTURES {
        let root: PathBuf = fixture.write(base.path()).unwrap();
        for ticked in LangSet::non_empty() {
            let failures: Vec<String> = check(fixture, &root, ticked);
            passed = passed.saturating_add(usize::from(failures.is_empty()));
            wrong.extend(failures);
        }
    }
    eprintln!("bb_lang_matrix passed={passed} of 49");
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert_eq!(passed, 49);
}

#[test]
fn detect_finds_each_fixture_s_languages_and_skips_dot_and_ignored_folders() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    for fixture in &FIXTURES {
        let root: PathBuf = fixture.write(base.path()).unwrap();
        let found: bb_lang::Detection = detect(&root, DETECT_BUDGET);
        assert_eq!(found.found, LangSet::of(fixture.langs), "{}", fixture.name);
        assert!(found.complete, "{}", fixture.name);
    }
}

#[test]
fn nothing_ticked_starts_nothing_shows_no_debug_and_colours_nothing() {
    let base: tempfile::TempDir = tempfile::tempdir().unwrap();
    for fixture in &FIXTURES {
        let root: PathBuf = fixture.write(base.path()).unwrap();
        let langs: Langs = Langs::new(LangSet::EMPTY, detect(&root, DETECT_BUDGET).found);
        assert!(langs.tools_at_open().is_empty(), "{}", fixture.name);
        assert!(!langs.debug_visible(), "{}", fixture.name);
        assert_eq!(langs.status(), "", "{}", fixture.name);
        for (file, _) in fixture.sources() {
            assert_eq!(langs.file_mode(&root.join(file)), FileMode::Plain);
        }
    }
}

#[test]
fn a_ticked_language_the_folder_lacks_starts_nothing_and_says_so() {
    let rust: LangSet = LangSet::of(&[Lang::Rust]);
    let csharp: LangSet = LangSet::of(&[Lang::CSharp]);
    let langs: Langs = Langs::new(rust, csharp);
    assert_eq!(langs.tools_at_open().len(), 0);
    assert!(!langs.debug_visible());
    assert_eq!(langs.status(), "No Rust project here");
    let both: Langs = Langs::new(csharp.with(Lang::Python), csharp);
    assert_eq!(both.status(), "C# · No Python project here");
}

#[test]
fn every_tool_has_its_language_in_the_table() {
    let listed: Vec<Tool> = SERVES
        .iter()
        .map(|(tool, _): &(Tool, Lang)| *tool)
        .collect();
    assert_eq!(listed, Tool::ALL.to_vec());
    for (tool, lang) in SERVES {
        assert_eq!(tool.lang(), lang, "{tool:?}");
    }
}

#[test]
fn csharp_only_costs_what_it_did_before_the_picker() {
    let langs: Langs = Langs::csharp_only();
    assert_eq!(langs.tools_at_open(), vec![Tool::Roslyn]);
    assert!(langs.debug_visible());
    assert_eq!(
        langs.file_mode(std::path::Path::new("App/appsettings.json")),
        FileMode::Colours(Lang::CSharp)
    );
    assert_eq!(langs.status(), "C#");
}

#[test]
fn a_folder_with_no_language_starts_with_csharp_ticked_and_others_with_what_they_hold() {
    assert_eq!(
        bb_lang::default_ticks(LangSet::EMPTY),
        LangSet::of(&[Lang::CSharp])
    );
    for set in LangSet::non_empty() {
        assert_eq!(bb_lang::default_ticks(set), set);
    }
}

#[test]
fn a_project_is_a_csproj_cargo_toml_pyproject_or_setup_py_never_a_solution() {
    let cases: [(&str, Option<Lang>); 7] = [
        ("Core/Core.csproj", Some(Lang::CSharp)),
        ("alpha/Cargo.toml", Some(Lang::Rust)),
        ("pyproject.toml", Some(Lang::Python)),
        ("tool/setup.py", Some(Lang::Python)),
        ("App.slnx", None),
        ("App.sln", None),
        ("requirements-dev.txt", None),
    ];
    for (file, lang) in cases {
        assert_eq!(Lang::of_project(&PathBuf::from(file)), lang, "{file}");
    }
}
