use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::{search, Found, Hit, Query, SearchError};

fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        let path: PathBuf = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

fn run(root: &Path, query: &str) -> Found {
    let stop: AtomicBool = AtomicBool::new(false);
    search(root, &Query::parse(query).unwrap(), 2_000, &stop)
}

fn names(found: &Found, root: &Path) -> Vec<String> {
    found
        .hits
        .iter()
        .map(|h: &Hit| {
            format!(
                "{}:{}",
                h.path.strip_prefix(root).unwrap().display(),
                h.line
            )
        })
        .collect()
}

#[test]
fn plain_text_matches_every_line_in_name_order_with_the_utf16_column() {
    let dir: tempfile::TempDir = tree(&[
        ("B.cs", "class B {}\n  // greet ÿou\n"),
        ("A.cs", "x\nvar s = \"é greet\";\n"),
    ]);
    let found: Found = run(dir.path(), "greet");
    assert_eq!(names(&found, dir.path()), ["A.cs:1", "B.cs:1"]);
    let first: &Hit = found.hits.first().unwrap();
    assert_eq!(first.column, 11, "é is one UTF-16 unit");
    assert_eq!(first.preview, "var s = \"é greet\";");
    assert_eq!(found.files, 2);
}

#[test]
fn smart_case_and_regex_queries() {
    let dir: tempfile::TempDir = tree(&[("A.cs", "Greet\ngreet\nGreeter\n")]);
    assert_eq!(run(dir.path(), "greet").hits.len(), 3);
    assert_eq!(
        names(&run(dir.path(), "Greet"), dir.path()),
        ["A.cs:0", "A.cs:2"]
    );
    assert_eq!(names(&run(dir.path(), "/^Greet$/"), dir.path()), ["A.cs:0"]);
    assert_eq!(
        run(dir.path(), "/^greet$/").hits.len(),
        2,
        "smart case in a regex too"
    );
    assert!(matches!(Query::parse(""), Err(SearchError::EmptyQuery)));
    assert!(matches!(Query::parse("//"), Err(SearchError::EmptyQuery)));
    assert!(matches!(
        Query::parse("/(/"),
        Err(SearchError::BadPattern(_))
    ));
    assert_eq!(run(dir.path(), "(").hits.len(), 0, "plain text is escaped");
}

#[test]
fn build_vcs_ide_ignored_and_binary_files_are_skipped_but_dot_folders_are_searched() {
    let dir: tempfile::TempDir = tree(&[
        ("Core/A.cs", "needle\n"),
        (".github/workflows/ci.yml", "needle\n"),
        (".config/dotnet-tools.json", "needle\n"),
        (".idea/workspace.xml", "needle\n"),
        (".DS_Store", "needle\n"),
        ("Core/obj/Gen.cs", "needle\n"),
        ("Core/bin/Debug/x.cs", "needle\n"),
        ("node_modules/m.js", "needle\n"),
        (".git/HEAD", "needle\n"),
        (".vs/cache", "needle\n"),
        (".gitignore", "secret/\n*.log\n"),
        ("secret/s.cs", "needle\n"),
        ("run.log", "needle\n"),
        ("blob.dll", "needle\0\0\0"),
    ]);
    assert_eq!(
        names(&run(dir.path(), "needle"), dir.path()),
        [
            ".config/dotnet-tools.json:0",
            ".github/workflows/ci.yml:0",
            "Core/A.cs:0"
        ],
        "tracked dot-folders are searched; .git, .vs, .idea, .DS_Store are not"
    );
}

#[test]
fn the_cap_stops_the_search_and_says_so() {
    let text: String = "hit\n".repeat(30);
    let dir: tempfile::TempDir = tree(&[("A.cs", &text), ("B.cs", &text)]);
    let stop: AtomicBool = AtomicBool::new(false);
    let found: Found = search(dir.path(), &Query::parse("hit").unwrap(), 40, &stop);
    assert_eq!(found.hits.len(), 40);
    assert!(found.capped);
    assert!(!run(dir.path(), "hit").capped);
}

#[test]
fn a_set_stop_flag_ends_the_search_early() {
    let dir: tempfile::TempDir = tree(&[("A.cs", "hit\n"), ("B.cs", "hit\n")]);
    let stop: AtomicBool = AtomicBool::new(true);
    let found: Found = search(dir.path(), &Query::parse("hit").unwrap(), 10, &stop);
    assert!(found.stopped);
    assert_eq!(found.hits, Vec::<Hit>::new());
}

#[test]
fn a_long_line_preview_is_shortened_on_a_char_boundary() {
    let line: String = format!("{}needle{}", "é".repeat(150), "x".repeat(300));
    let dir: tempfile::TempDir = tree(&[("A.cs", &line)]);
    let found: Found = run(dir.path(), "needle");
    let preview: &str = &found.hits.first().unwrap().preview;
    assert!(preview.chars().count() <= 201, "{}", preview.len());
    assert!(preview.ends_with('…'));
}

#[test]
fn anchors_match_at_every_line_end_including_crlf_files() {
    let dir: tempfile::TempDir = tree(&[
        ("Crlf.cs", "class A\r\n{\r\n    greet\r\n}\r\n"),
        ("Lf.cs", "x\ngreet\n"),
    ]);
    assert_eq!(
        names(&run(dir.path(), "/^\\s*greet$/"), dir.path()),
        ["Crlf.cs:2", "Lf.cs:1"]
    );
}
