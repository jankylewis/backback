//! BBEOD09: git decides what the Explorer shows as ignored (pink).

#[cfg(test)]
mod git {
    use std::fs;
    use std::path::Path;

    use bb_explorer::{Entry, Explorer};

    fn child<'a>(entry: &'a Entry, name: &str) -> &'a Entry {
        entry
            .children()
            .iter()
            .find(|c: &&Entry| c.name() == name)
            .unwrap_or_else(|| panic!("no {name}"))
    }

    fn repo(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        for (path, text) in files {
            let full: std::path::PathBuf = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        dir
    }

    fn open(root: &Path) -> Explorer {
        Explorer::open(root).unwrap()
    }

    #[test]
    fn a_gitignored_file_is_ignored_and_a_tracked_file_is_not() {
        let dir: tempfile::TempDir = repo(&[
            (".gitignore", ".env.*\n"),
            (".env.dev_db", "x"),
            ("Client.cs", ""),
        ]);
        let tree: Explorer = open(dir.path());
        assert!(child(tree.root(), ".env.dev_db").ignored());
        assert!(!child(tree.root(), "Client.cs").ignored());
    }

    #[test]
    fn an_ignored_folder_makes_its_whole_subtree_ignored() {
        let dir: tempfile::TempDir = repo(&[(".gitignore", "bin/\n"), ("bin/Debug/a.dll", "")]);
        let mut tree: Explorer = open(dir.path());
        assert!(child(tree.root(), "bin").ignored());
        tree.toggle(&dir.path().join("bin")).unwrap();
        let bin: &Entry = child(tree.root(), "bin");
        assert!(
            child(bin, "Debug").ignored(),
            "inherits from the ignored folder"
        );
    }

    #[test]
    fn a_nested_gitignore_and_a_negated_pattern_are_honoured() {
        let dir: tempfile::TempDir = repo(&[
            (".gitignore", "*.log\n!keep.log\n"),
            ("Sub/.gitignore", "local.txt\n"),
            ("Sub/local.txt", ""),
            ("Sub/other.txt", ""),
            ("run.log", ""),
            ("keep.log", ""),
        ]);
        let mut tree: Explorer = open(dir.path());
        assert!(child(tree.root(), "run.log").ignored());
        assert!(!child(tree.root(), "keep.log").ignored(), "!keep.log");
        tree.toggle(&dir.path().join("Sub")).unwrap();
        let sub: &Entry = child(tree.root(), "Sub");
        assert!(child(sub, "local.txt").ignored());
        assert!(!child(sub, "other.txt").ignored());
    }

    #[test]
    fn git_info_exclude_is_honoured() {
        let dir: tempfile::TempDir =
            repo(&[(".git/info/exclude", "scratch.md\n"), ("scratch.md", "")]);
        let tree: Explorer = open(dir.path());
        assert!(child(tree.root(), "scratch.md").ignored());
    }

    #[test]
    fn outside_a_git_repo_nothing_is_ignored() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
        fs::write(dir.path().join("run.log"), "").unwrap();
        let tree: Explorer = open(dir.path());
        assert!(!child(tree.root(), "run.log").ignored());
    }
}
