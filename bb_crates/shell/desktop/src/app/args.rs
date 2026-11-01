//! The command line: a folder, a solution or a file, and `--no-write`.

use std::path::PathBuf;

/// What the command line asks for.
pub(crate) struct Start {
    pub(crate) root: PathBuf,
    pub(crate) file: Option<PathBuf>,
    pub(crate) no_write: bool,
}

pub(crate) fn parse_args() -> Start {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let no_write: bool = args.iter().any(|a: &std::ffi::OsString| a == "--no-write");
    let argument: PathBuf = args
        .iter()
        .find(|a: &&std::ffi::OsString| *a != "--no-write")
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let (root, file): (PathBuf, Option<PathBuf>) = split_argument(argument);
    let root: PathBuf = root.canonicalize().unwrap_or(root);
    Start {
        root,
        file,
        no_write,
    }
}

/// A file argument opens the nearest folder above it that holds a solution
/// (else its own folder) in the Explorer, and the file in a tab; a solution
/// file opens its folder.
fn split_argument(argument: PathBuf) -> (PathBuf, Option<PathBuf>) {
    if !argument.is_file() {
        return (argument, None);
    }
    let folder: PathBuf = argument
        .parent()
        .filter(|parent: &&std::path::Path| !parent.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
    let solution: bool = argument
        .extension()
        .is_some_and(|ext: &std::ffi::OsStr| ext == "slnx" || ext == "sln");
    if solution {
        return (folder, None);
    }
    let root: PathBuf = folder
        .ancestors()
        .find(|dir: &&std::path::Path| crate::intel::bb_worker::find_solution(dir).is_some())
        .map_or(folder.clone(), std::path::Path::to_path_buf);
    (root, Some(argument))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
