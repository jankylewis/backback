//! A built copy of `fixtures/_Harness`, shared by the tests of one process.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bb_dotnet::ProjectPath;
use bb_testkit::{ApexBackend, ApexSettings, CommandLine, Workspace};

/// The Apex CLI version `MbTx` pins.
pub const APEX_CLI: &str = "0.8.2-pre";

static BUILT: OnceLock<PathBuf> = OnceLock::new();

/// A fresh copy of the harness under the temp folder.
pub fn copy(name: &str) -> PathBuf {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("bb-testkit-{name}-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    let source: PathBuf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../shell/desktop/fixtures/_Harness");
    copy_tree(&source, &dir);
    std::fs::canonicalize(dir).unwrap()
}

/// The shared copy with both test projects built (Debug).
pub fn built() -> &'static Path {
    BUILT.get_or_init(|| {
        build_copy(
            "e2e",
            &[
                "_NUnitApex/_NUnitApex.csproj",
                "XunitPlaywright/_XunitPlaywright.csproj",
            ],
        )
    })
}

/// A fresh copy named `name` with `projects` built (Debug).
pub fn build_copy(name: &str, projects: &[&str]) -> PathBuf {
    let dir: PathBuf = copy(name);
    for project in projects {
        let status: std::process::ExitStatus = std::process::Command::new("dotnet")
            .args(["build", project, "-c", "Debug", "-nologo", "-v", "q"])
            .current_dir(&dir)
            .status()
            .unwrap();
        assert!(status.success(), "build of {project} failed");
    }
    dir
}

/// Processes whose command line contains `text` (`pgrep -f`).
pub fn processes_with(text: &str) -> Vec<String> {
    let out: std::process::Output = std::process::Command::new("pgrep")
        .args(["-fl", text])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A project of a harness copy.
pub fn project(root: &Path, relative: &str) -> ProjectPath {
    ProjectPath::from_path(root.join(relative))
}

/// The Apex CLI: `BB_APEX_COMMAND`, else the pinned tool in the NuGet cache.
pub fn apex_command() -> CommandLine {
    if let Ok(given) = std::env::var("BB_APEX_COMMAND") {
        return CommandLine::parse(&given).unwrap();
    }
    let home: PathBuf = PathBuf::from(std::env::var("HOME").unwrap());
    let dll: PathBuf = home.join(format!(
        ".nuget/packages/apex.cli/{APEX_CLI}/tools/net10.0/any/apex.dll"
    ));
    assert!(
        dll.is_file(),
        "apex.cli {APEX_CLI} is not installed: {}",
        dll.display()
    );
    CommandLine::parse(&format!("dotnet {}", dll.display())).unwrap()
}

/// An Apex backend on `root` that does not build first.
pub fn apex(root: &Path) -> ApexBackend {
    let settings: ApexSettings =
        ApexSettings::parse(r#"{ "apexTestkit.buildBeforeRun": false }"#, root).unwrap();
    ApexBackend::new(Workspace::at(root.to_path_buf()), settings)
        .unwrap()
        .with_command(apex_command())
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        let target: PathBuf = to.join(path.file_name().unwrap());
        let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() && name != "obj" && name != "bin" {
            copy_tree(&path, &target);
        } else if path.is_file() {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}
