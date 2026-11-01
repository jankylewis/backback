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

/// A stand-in project in `temp` that names `assembly` as its `TargetPath`.
///
/// The Apex CLI lists a project by `msbuild -getProperty:TargetPath`. A copy
/// of the real project file loses its import chain (`Directory.Build.props`
/// and the rest), so the caller evaluates the real project in place and
/// hands the CLI this file. It has no SDK, so it imports nothing.
pub(crate) fn stub_project(
    project: &Path,
    assembly: &Path,
    temp: &TempDir,
) -> Result<PathBuf, TestkitError> {
    let name: &std::ffi::OsStr = project
        .file_name()
        .ok_or_else(|| io_error(project, std::io::ErrorKind::InvalidInput))?;
    let folder: PathBuf = temp.path().join("project");
    std::fs::create_dir_all(&folder).map_err(|error: std::io::Error| TestkitError::Io {
        path: folder.clone(),
        source: error,
    })?;
    let stub: PathBuf = folder.join(name);
    let text: String = format!(
        "<Project>\n  <PropertyGroup>\n    <TargetPath>{}</TargetPath>\n  </PropertyGroup>\n</Project>\n",
        property_text(&assembly.to_string_lossy())
    );
    std::fs::write(&stub, text).map_err(|error: std::io::Error| TestkitError::Io {
        path: stub.clone(),
        source: error,
    })?;
    Ok(stub)
}

/// `value` as literal MSBuild property text inside XML: MSBuild's own
/// specials become `%HH`, then XML specials become entities.
fn property_text(value: &str) -> String {
    let mut out: String = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '%' => out.push_str("%25"),
            '$' => out.push_str("%24"),
            '@' => out.push_str("%40"),
            '\'' => out.push_str("%27"),
            ';' => out.push_str("%3B"),
            '?' => out.push_str("%3F"),
            '*' => out.push_str("%2A"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
    out
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
    #[test]
    fn msbuild_reads_the_stub_target_path_back_unchanged() {
        let temp: TempDir = TempDir::new().unwrap();
        let assembly: PathBuf = temp
            .path()
            .join("odd $(Home) @(Items) %(Meta) 50%; a&b <c> 'd' ?*")
            .join("Sample.dll");
        let stub: PathBuf =
            stub_project(Path::new("/any/where/Sample.csproj"), &assembly, &temp).unwrap();
        let evaluated: std::process::Output = std::process::Command::new("dotnet")
            .arg("msbuild")
            .arg(&stub)
            .args(["-getProperty:TargetPath", "-nologo"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(evaluated.status.success(), "{evaluated:?}");
        let printed: String = String::from_utf8(evaluated.stdout).unwrap();
        assert_eq!(printed.trim(), assembly.to_string_lossy());
        assert_eq!(stub.file_name().unwrap(), "Sample.csproj");
    }
}
