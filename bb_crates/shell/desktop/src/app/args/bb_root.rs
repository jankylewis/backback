//! Which folder a file argument opens (BBEOD33, BBEOD32 row 3): the nearest
//! project root of the file's language, then the outermost root of another
//! language that holds it. The walk never reaches the home folder and ends
//! at a git root.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_lang::Lang;

/// How long `cargo metadata` may take to name a Rust workspace root when a
/// window opens (a broken `Cargo.toml` fails at once).
pub(crate) const CARGO_ROOT_DEADLINE: Duration = Duration::from_secs(5);

/// The Cargo workspace root for a folder holding a `Cargo.toml`, if cargo
/// names one.
pub(crate) type CargoRoot<'a> = &'a dyn Fn(&Path) -> Option<PathBuf>;

/// The real seam: `cargo metadata`'s `workspace_root`.
pub(crate) fn cargo_root(dir: &Path) -> Option<PathBuf> {
    bb_cargo::members(dir, CARGO_ROOT_DEADLINE)
        .ok()
        .map(|workspace: bb_cargo::CargoWorkspace| workspace.root)
}

/// The folders a file in `folder` may root at, nearest first: `folder` and
/// its parents, up to and with the first git root, never the home folder or
/// above it.
pub(crate) fn walk(folder: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    for dir in folder.ancestors() {
        if home.is_some_and(|home: &Path| dir == home) || dir.as_os_str().is_empty() {
            break;
        }
        dirs.push(dir.to_path_buf());
        if dir.join(".git").exists() {
            break;
        }
    }
    dirs
}

/// Whether `dir` is a project root of `lang`: a solution, a `Cargo.toml`,
/// or a `pyproject.toml` / `setup.py`.
pub(crate) fn is_root_of(dir: &Path, lang: Lang) -> bool {
    match lang {
        Lang::CSharp => crate::intel::bb_worker::find_solution(dir).is_some(),
        Lang::Rust => dir.join("Cargo.toml").is_file(),
        Lang::Python => dir.join("pyproject.toml").is_file() || dir.join("setup.py").is_file(),
    }
}

/// The folder a `file` argument opens; None: its own folder.
///
/// A C# file, or a file of no language, roots at the nearest solution as
/// before. A Rust file roots at its Cargo workspace (cargo's answer when it
/// lies inside the walk, else the nearest `Cargo.toml`); a Python file at
/// the nearest `pyproject.toml` or `setup.py`. Then a mixed workspace: the
/// outermost folder in the walk holding another language's root.
pub(crate) fn project_root(file: &Path, home: Option<&Path>, cargo: CargoRoot) -> Option<PathBuf> {
    let folder: &Path = file.parent()?;
    let dirs: Vec<PathBuf> = walk(folder, home);
    let lang: Lang = Lang::of_source(file).unwrap_or(Lang::CSharp);
    let nearest: usize = dirs
        .iter()
        .position(|dir: &PathBuf| is_root_of(dir, lang))?;
    let own: usize = match lang {
        Lang::Rust => dirs
            .get(nearest)
            .and_then(|dir: &PathBuf| cargo(dir))
            .and_then(|root: PathBuf| same_as(&dirs, &root))
            .filter(|at: &usize| *at >= nearest)
            .unwrap_or(nearest),
        Lang::CSharp | Lang::Python => nearest,
    };
    let outer: Option<usize> = dirs.iter().rposition(|dir: &PathBuf| {
        Lang::ALL
            .into_iter()
            .any(|other: Lang| other != lang && is_root_of(dir, other))
    });
    let at: usize = outer.filter(|at: &usize| *at > own).unwrap_or(own);
    dirs.get(at).cloned()
}

/// The index in `dirs` of the folder `root` names (cargo prints canonical
/// paths; the walk keeps the argument's spelling).
fn same_as(dirs: &[PathBuf], root: &Path) -> Option<usize> {
    let wanted: PathBuf = root.canonicalize().ok()?;
    dirs.iter().position(|dir: &PathBuf| {
        dir.canonicalize()
            .is_ok_and(|canonical: PathBuf| canonical == wanted)
    })
}
