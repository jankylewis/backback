//! Start the real server, not the dotnet tool's launcher. The launcher
//! (`roslyn-language-server` in the tools folder) only starts the server
//! from the tool store and then waits as its parent for the whole session:
//! about 49 MB that does nothing (BB-PRF-005). The real server is used only
//! when the store next to the launcher holds exactly one server for this
//! platform; anything else keeps the launcher, as before.

use std::path::{Path, PathBuf};

use tokio::process::Command;

/// The server's file name inside the tool package.
const SERVER: &str = "Microsoft.CodeAnalysis.LanguageServer";

/// The command that starts `launcher`'s server: the real server when it is
/// found, else the launcher itself.
pub(crate) fn command(launcher: &Path) -> Command {
    let Some(server): Option<PathBuf> = server_behind(launcher) else {
        return Command::new(launcher);
    };
    let mut cmd: Command = Command::new(server);
    // The launcher tells the server where .NET is; without it, the server
    // looks only in the default place. A user value wins.
    if std::env::var_os("DOTNET_ROOT").is_none() {
        if let Some(root) = dotnet_root() {
            let _set: &mut Command = cmd.env("DOTNET_ROOT", root);
        }
    }
    cmd
}

/// The server behind `launcher`: `<tools>/.store/<name>/<version>/<name>.<rid>/
/// <version>/tools/<tfm>/<rid>/Microsoft.CodeAnalysis.LanguageServer`, when
/// exactly one exists.
pub(crate) fn server_behind(launcher: &Path) -> Option<PathBuf> {
    let tools: &Path = launcher.parent()?;
    let name: String = launcher.file_name()?.to_str()?.to_owned();
    let rid: &str = rid()?;
    let found: Vec<PathBuf> = folders(&tools.join(".store").join(&name))
        .into_iter()
        .flat_map(|version: PathBuf| {
            let tag: std::ffi::OsString = version.file_name().unwrap_or_default().to_owned();
            folders(
                &version
                    .join(format!("{name}.{rid}"))
                    .join(tag)
                    .join("tools"),
            )
        })
        .map(|framework: PathBuf| framework.join(rid).join(SERVER))
        .filter(|server: &PathBuf| server.is_file())
        .collect();
    match found.as_slice() {
        [one] => Some(one.clone()),
        _ => None,
    }
}

/// This build's platform, as the tool package names it.
fn rid() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("osx-arm64"),
        ("macos", "x86_64") => Some("osx-x64"),
        ("linux", "x86_64") => Some("linux-x64"),
        ("linux", "aarch64") => Some("linux-arm64"),
        _ => None,
    }
}

/// The folders in `dir`, in name order; none when it cannot be read.
fn folders(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries: std::fs::ReadDir| {
            entries
                .flatten()
                .map(|entry: std::fs::DirEntry| entry.path())
                .filter(|path: &PathBuf| path.is_dir())
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

/// The folder of the `dotnet` on `PATH`, links resolved.
fn dotnet_root() -> Option<PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir: PathBuf| dir.join("dotnet"))
        .find(|dotnet: &PathBuf| dotnet.is_file())
        .and_then(|dotnet: PathBuf| dotnet.canonicalize().ok())
        .and_then(|dotnet: PathBuf| dotnet.parent().map(Path::to_path_buf))
}

#[cfg(test)]
mod tests {
    use super::{rid, server_behind, SERVER};
    use std::path::{Path, PathBuf};

    /// A tools folder with a launcher and `versions` installed servers.
    fn tools(dir: &Path, versions: &[&str]) -> PathBuf {
        let launcher: PathBuf = dir.join("roslyn-language-server");
        std::fs::write(&launcher, "").unwrap();
        let rid: &str = rid().unwrap();
        for version in versions {
            let server: PathBuf = dir
                .join(".store/roslyn-language-server")
                .join(version)
                .join(format!("roslyn-language-server.{rid}"))
                .join(version)
                .join("tools/net10.0")
                .join(rid)
                .join(SERVER);
            std::fs::create_dir_all(server.parent().unwrap()).unwrap();
            std::fs::write(&server, "").unwrap();
        }
        launcher
    }

    #[test]
    fn one_installed_server_is_started_directly() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let launcher: PathBuf = tools(dir.path(), &["5.13.0"]);
        let server: PathBuf = server_behind(&launcher).unwrap();
        assert!(server.ends_with(Path::new("tools/net10.0").join(rid().unwrap()).join(SERVER)));
    }

    #[test]
    fn none_or_two_servers_keep_the_launcher() {
        let empty: tempfile::TempDir = tempfile::tempdir().unwrap();
        assert_eq!(server_behind(&tools(empty.path(), &[])), None);
        let two: tempfile::TempDir = tempfile::tempdir().unwrap();
        assert_eq!(
            server_behind(&tools(two.path(), &["5.12.0", "5.13.0"])),
            None
        );
    }
}
