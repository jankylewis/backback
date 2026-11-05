//! The three views on a throwaway workspace, and the map of this repo
//! checked against a plain line scan so no public item goes missing.

use std::path::{Path, PathBuf};

use bb_kmap::{find, index, outline, rust_files, scan, FileMap, Item};

#[cfg(test)]
mod helpers {
    use std::path::PathBuf;

    pub struct Scratch(pub PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir: PathBuf =
                std::env::temp_dir().join(format!("bb-kmap-{tag}-{}", std::process::id()));
            let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
            Self(dir)
        }

        fn file(&self, rel: &str, text: &str) {
            let path: PathBuf = self.0.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _gone: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
        }
    }

    pub fn workspace(tag: &str) -> Scratch {
        let s: Scratch = Scratch::new(tag);
        s.file(
            "Cargo.toml",
            "[workspace]\nmembers = [\n  \"a\",\n  \"b\",\n]\n",
        );
        s.file(
            "a/Cargo.toml",
            "[package]\nname = \"bb-a\"\ndescription = \"Crate A\"\n",
        );
        s.file(
            "b/Cargo.toml",
            "[package]\nname = \"bb-b\"\n\n[dependencies]\nbb-a.workspace = true\nserde = \"1\"\n\n[dev-dependencies]\nbb-c.workspace = true\n",
        );
        s.file("a/src/lib.rs", "//! A lib.\n\npub fn shown() {}\nfn hidden() {}\n\nstruct Quiet;\nimpl Quiet {\n    fn inner(&self) {}\n}\n");
        s.file("b/src/main.rs", "fn main() {}\n");
        s.file("b/tests/it.rs", "#[test]\nfn one() {}\n");
        s.file("b/target/junk.rs", "pub fn never() {}\n");
        s
    }
}

use helpers::{workspace, Scratch};

#[test]
fn the_index_lists_each_member_with_its_internal_uses() {
    let s: Scratch = workspace("index");
    let text: String = index(&s.0).unwrap();
    assert!(text.starts_with("2 crates, 3 files,"), "{text}");
    assert!(
        text.contains("bb-a a (1 files, 9 lines) uses: -\n  Crate A\n"),
        "{text}"
    );
    assert!(
        text.contains("bb-b b (2 files, 3 lines) uses: bb-a\n"),
        "dev deps and serde stay out: {text}"
    );
}

#[test]
fn the_outline_shows_public_items_and_all_adds_private_ones() {
    let s: Scratch = workspace("outline");
    let text: String = outline(&s.0.join("a"), false).unwrap();
    assert!(
        text.contains("lib.rs (9 lines) A lib.\n  3 pub fn shown()\n"),
        "{text}"
    );
    assert!(
        !text.contains("hidden") && !text.contains("Quiet"),
        "{text}"
    );
    let all: String = outline(&s.0.join("a"), true).unwrap();
    assert!(
        all.contains("  4 fn hidden()\n") && all.contains("    8 fn inner(&self)\n"),
        "{all}"
    );
    let tests: String = outline(&s.0.join("b"), false).unwrap();
    assert!(tests.contains("it.rs (2 lines) \n  tests: 1\n"), "{tests}");
    assert!(!tests.contains("never"), "target/ is skipped: {tests}");
}

#[test]
fn find_matches_a_name_or_owner_and_name_at_any_visibility() {
    let s: Scratch = workspace("find");
    assert!(find(&s.0, "hidden")
        .unwrap()
        .contains("lib.rs:4 fn hidden()"));
    assert!(find(&s.0, "Quiet::inner")
        .unwrap()
        .contains("lib.rs:8 fn inner(&self)"));
    assert_eq!(find(&s.0, "nothing").unwrap(), "no item named nothing\n");
}

const HEADS: [&str; 6] = [
    "pub fn ",
    "pub(crate) fn ",
    "pub struct ",
    "pub(crate) struct ",
    "pub enum ",
    "pub trait ",
];

#[test]
fn every_public_head_in_this_repo_is_on_the_map() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut checked: usize = 0;
    for file in rust_files(&root).unwrap() {
        let src: String = std::fs::read_to_string(&file).unwrap();
        let tests_inside: bool =
            src.contains("#[cfg(test)]\nmod tests {") || src.contains("macro_rules!");
        let test_file: bool = file
            .components()
            .any(|c: std::path::Component<'_>| c.as_os_str() == "tests");
        if tests_inside || test_file || file.ends_with("tests.rs") {
            continue;
        }
        let map: FileMap = scan(&src);
        let lines: Vec<&str> = src.lines().collect();
        for (at, line) in lines.iter().enumerate() {
            let head: &str = line.trim_start();
            let test_only: bool = at > 0 && lines[at - 1].trim() == "#[cfg(test)]";
            if !test_only && HEADS.iter().any(|h: &&str| head.starts_with(h)) {
                checked += 1;
                let found: bool = map.items.iter().any(|i: &Item| i.line == at + 1);
                assert!(found, "{}:{} missing: {head}", file.display(), at + 1);
            }
        }
    }
    assert!(checked > 500, "the scan looked at too little: {checked}");
}
