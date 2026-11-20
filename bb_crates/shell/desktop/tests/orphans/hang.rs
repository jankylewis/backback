//! A C# project whose build runs a long step (`Exec sleep 600`), picked as
//! C# only. Build > Build Solution runs `dotnet build` in an owned group;
//! nothing in it reads from backback, so when backback is killed only the
//! group's end stops the build and its step (BBEOD42).

use std::path::{Path, PathBuf};

/// The long step, as `ps` shows it.
pub const STEP: &str = "sleep 600";

/// A slow runner's build: this long before the long step starts (CI run
/// 44768652825 crashed backback before `sleep 600` began, BB-GAT-020).
pub const SLOW_START: &str = "sleep 12";

pub fn fixture(base: &Path, slow: bool) -> (PathBuf, PathBuf) {
    let before: String = if slow {
        format!("/bin/{SLOW_START}; ")
    } else {
        String::new()
    };
    let root: PathBuf = base.join("hang");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("Hang.slnx"),
        "<Solution>\n  <Project Path=\"Hang.csproj\" />\n</Solution>\n",
    )
    .unwrap();
    std::fs::write(
        root.join("Hang.csproj"),
        format!(
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup><Target Name="LongStep" BeforeTargets="Build"><Exec Command="{before}/bin/{STEP}" /></Target></Project>"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("Program.cs"),
        "class Program { static void Main() { } }\n",
    )
    .unwrap();
    let root: PathBuf = root.canonicalize().unwrap();
    let state: PathBuf = base.join("state");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("langs"),
        format!("backback langs v2\npicked\tcsharp\t{}\n", root.display()),
    )
    .unwrap();
    (root.join("Program.cs"), state)
}
