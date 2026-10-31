//! Copy the built test output to temp before discovery starts a test host.

use std::path::{Path, PathBuf};

use crate::backend::TempDir;
use crate::TestkitError;

pub(crate) fn assembly(assembly: &Path, temp: &TempDir) -> Result<PathBuf, TestkitError> {
    let source: &Path = assembly
        .parent()
        .ok_or_else(|| io_error(assembly, std::io::ErrorKind::InvalidInput))?;
    let name: &std::ffi::OsStr = assembly
        .file_name()
        .ok_or_else(|| io_error(assembly, std::io::ErrorKind::InvalidInput))?;
    let output: PathBuf = temp.path().join("build");
    copy_tree(source, &output)?;
    Ok(output.join(name))
}

pub(crate) fn project(project: &Path, temp: &TempDir) -> Result<PathBuf, TestkitError> {
    let source: &Path = project
        .parent()
        .ok_or_else(|| io_error(project, std::io::ErrorKind::InvalidInput))?;
    let name: &std::ffi::OsStr = project
        .file_name()
        .ok_or_else(|| io_error(project, std::io::ErrorKind::InvalidInput))?;
    let output: PathBuf = temp.path().join("project");
    std::fs::create_dir_all(&output).map_err(|error: std::io::Error| TestkitError::Io {
        path: output.clone(),
        source: error,
    })?;
    std::fs::copy(project, output.join(name)).map_err(|error: std::io::Error| {
        TestkitError::Io {
            path: project.to_path_buf(),
            source: error,
        }
    })?;
    copy_tree(&source.join("bin"), &output.join("bin"))?;
    Ok(output.join(name))
}

pub(crate) fn original_assembly(
    project: &Path,
    safe_project: &Path,
    assembly: &Path,
) -> Result<PathBuf, TestkitError> {
    let original: &Path = project
        .parent()
        .ok_or_else(|| io_error(project, std::io::ErrorKind::InvalidInput))?;
    let shadow: &Path = safe_project
        .parent()
        .ok_or_else(|| io_error(safe_project, std::io::ErrorKind::InvalidInput))?;
    let relative: &Path =
        assembly
            .strip_prefix(shadow)
            .map_err(|_: std::path::StripPrefixError| {
                io_error(assembly, std::io::ErrorKind::InvalidInput)
            })?;
    Ok(original.join(relative))
}

fn copy_tree(source: &Path, output: &Path) -> Result<(), TestkitError> {
    std::fs::create_dir_all(output).map_err(|error: std::io::Error| TestkitError::Io {
        path: output.to_path_buf(),
        source: error,
    })?;
    for found in std::fs::read_dir(source).map_err(|error: std::io::Error| TestkitError::Io {
        path: source.to_path_buf(),
        source: error,
    })? {
        let entry: std::fs::DirEntry = found.map_err(|error: std::io::Error| TestkitError::Io {
            path: source.to_path_buf(),
            source: error,
        })?;
        let path: PathBuf = entry.path();
        let target: PathBuf = output.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else if runtime_asset(&path) {
            std::fs::copy(&path, &target).map_err(|error: std::io::Error| TestkitError::Io {
                path: target,
                source: error,
            })?;
        }
    }
    Ok(())
}

fn runtime_asset(path: &Path) -> bool {
    let Some(name): Option<&str> = path.file_name().and_then(std::ffi::OsStr::to_str) else {
        return false;
    };
    [
        ".dll",
        ".exe",
        ".pdb",
        ".deps.json",
        ".runtimeconfig.json",
        ".runtimeconfig.dev.json",
        ".so",
        ".dylib",
    ]
    .iter()
    .any(|suffix: &&str| name.ends_with(suffix))
}

fn io_error(path: &Path, kind: std::io::ErrorKind) -> TestkitError {
    TestkitError::Io {
        path: path.to_path_buf(),
        source: std::io::Error::from(kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_copies_only_runtime_assets() {
        let source: TempDir = TempDir::new().unwrap();
        let build: PathBuf = source.path().join("bin");
        std::fs::create_dir_all(&build).unwrap();
        for name in [
            "sample.dll",
            "sample.deps.json",
            "auth-store.json",
            "notes.txt",
        ] {
            std::fs::write(build.join(name), "marker").unwrap();
        }
        let target: TempDir = TempDir::new().unwrap();
        let copied: PathBuf = assembly(&build.join("sample.dll"), &target).unwrap();
        let folder: &Path = copied.parent().unwrap();
        assert!(copied.is_file());
        assert!(folder.join("sample.deps.json").is_file());
        assert!(!folder.join("auth-store.json").exists());
        assert!(!folder.join("notes.txt").exists());
    }
}
