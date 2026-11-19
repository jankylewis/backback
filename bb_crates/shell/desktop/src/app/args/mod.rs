//! The command line: a folder, a solution or a file, and `--no-write`.

use std::path::{Path, PathBuf};

pub(crate) mod bb_root;
#[cfg(test)]
mod root_tests;

/// A workspace to open: its Explorer root and the file to show.
pub(crate) struct Start {
    pub(crate) root: PathBuf,
    pub(crate) file: Option<PathBuf>,
    pub(crate) no_write: bool,
    /// The start page's language ticks; None: the saved choice, else what
    /// the folder holds (BBEOD26).
    pub(crate) ticked: Option<bb_lang::LangSet>,
}

/// What a launch opens: a workspace, or the start page (BBEOD23).
pub(crate) enum Launch {
    /// A folder, a solution or a file, from the argument or the working folder.
    Workspace(Start),
    /// A Finder or Dock launch with no argument.
    StartPage { no_write: bool },
}

impl Launch {
    /// `--no-write`: every save is locked.
    pub(crate) fn no_write(&self) -> bool {
        match self {
            Self::Workspace(start) => start.no_write,
            Self::StartPage { no_write } => *no_write,
        }
    }
}

pub(crate) fn parse_args() -> Launch {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let no_write: bool = args.iter().any(|a: &std::ffi::OsString| a == "--no-write");
    let given: Option<PathBuf> = args
        .iter()
        .find(|a: &&std::ffi::OsString| *a != "--no-write")
        .map(PathBuf::from);
    let cwd: PathBuf =
        std::env::current_dir().unwrap_or_else(|_: std::io::Error| PathBuf::from("/"));
    match resolve(given, &cwd) {
        Some(argument) => Launch::Workspace(start_for(argument, no_write)),
        None => Launch::StartPage { no_write },
    }
}

/// What to open: the argument; else the working folder, except for a Finder
/// or Dock launch (working folder `/`): None, the start page (BBEOD23;
/// BB-BUG-006: never the disk root).
pub(crate) fn resolve(given: Option<PathBuf>, cwd: &Path) -> Option<PathBuf> {
    given.or_else(|| (cwd != Path::new("/")).then(|| cwd.to_path_buf()))
}

/// The workspace for a folder, solution or file path.
pub(crate) fn start_for(argument: PathBuf, no_write: bool) -> Start {
    let (root, file): (PathBuf, Option<PathBuf>) = split_argument(argument);
    let root: PathBuf = root.canonicalize().unwrap_or(root);
    Start {
        root,
        file,
        no_write,
        ticked: None,
    }
}

/// A file argument opens its project's folder ([`bb_root::project_root`],
/// else its own folder) in the Explorer, and the file in a tab; a solution
/// file opens its folder.
fn split_argument(argument: PathBuf) -> (PathBuf, Option<PathBuf>) {
    if !argument.is_file() {
        return (argument, None);
    }
    let file: PathBuf =
        std::path::absolute(&argument).unwrap_or_else(|_: std::io::Error| argument.clone());
    let folder: PathBuf = file
        .parent()
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
    let solution: bool = argument
        .extension()
        .is_some_and(|ext: &std::ffi::OsStr| ext == "slnx" || ext == "sln");
    if solution {
        return (folder, None);
    }
    let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
    let root: PathBuf =
        bb_root::project_root(&file, home.as_deref(), &bb_root::cargo_root).unwrap_or(folder);
    (root, Some(argument))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB-BUG-006: a Finder or Dock launch runs in `/` with no argument; it
    /// shows the start page (BBEOD23), never the disk root.
    #[test]
    fn a_finder_launch_shows_the_start_page_and_never_opens_the_disk_root() {
        let slash: &Path = Path::new("/");
        assert_eq!(resolve(None, slash), None, "a Finder or Dock launch");
        let cwd: PathBuf = PathBuf::from("/w/Here");
        assert_eq!(resolve(None, &cwd), Some(cwd.clone()), "a terminal launch");
        let given: PathBuf = PathBuf::from("/w/Given.slnx");
        assert_eq!(resolve(Some(given.clone()), slash), Some(given));
    }

    #[test]
    fn a_file_argument_opens_its_folder_and_the_file() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let (folder, file): (PathBuf, PathBuf) =
            (dir.path().to_path_buf(), dir.path().join("A.cs"));
        std::fs::write(&file, "class A {}").unwrap();
        assert_eq!(split_argument(file.clone()), (folder.clone(), Some(file)));
        assert_eq!(split_argument(folder.clone()), (folder, None));
    }

    #[test]
    fn a_file_argument_roots_at_the_nearest_folder_with_a_solution() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let root: PathBuf = dir.path().to_path_buf();
        let file: PathBuf = root.join("Core/Deep/A.cs");
        std::fs::create_dir_all(root.join("Core/Deep")).unwrap();
        std::fs::write(&file, "class A {}").unwrap();
        std::fs::write(root.join("App.slnx"), "<Solution />").unwrap();
        assert_eq!(split_argument(file.clone()), (root.clone(), Some(file)));
        let solution: PathBuf = root.join("App.slnx");
        assert_eq!(split_argument(solution), (root, None));
    }

    /// BBEOD32 row 3: a Rust or Python file roots at its project, not at
    /// its own folder, in every fixture that holds its language.
    #[test]
    fn a_rust_or_python_file_argument_roots_at_its_project() {
        let base: tempfile::TempDir = tempfile::tempdir().unwrap();
        let mut wrong: Vec<String> = Vec::new();
        for fixture in &bb_lang::fixtures::FIXTURES {
            let root: PathBuf = base.path().join(fixture.name);
            let _written: PathBuf = fixture.write(base.path()).unwrap();
            for (source, lang) in fixture.sources() {
                let held: bool = fixture.langs.contains(&lang);
                if !held || lang == bb_lang::Lang::CSharp {
                    continue;
                }
                let file: PathBuf = root.join(source);
                let got: (PathBuf, Option<PathBuf>) = split_argument(file.clone());
                if got != (root.clone(), Some(file)) {
                    wrong.push(format!("{} {source}: {got:?}", fixture.name));
                }
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }
}
