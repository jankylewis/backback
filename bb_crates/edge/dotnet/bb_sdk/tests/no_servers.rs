//! A test-side `dotnet build` leaves no compiler server or MSBuild node
//! behind, and never hands its compile to one the user already runs (BBEOD43).

#[cfg(test)]
mod checks {
    use std::path::Path;
    use std::process::{Command, Output};

    use bb_dotnet::no_servers::{self, Ran};

    const SUITE_BUILD: [&str; 7] = ["build", "App.slnx", "-c", "Debug", "-nologo", "-v", "q"];

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Proc {
        pid: u32,
        group: u32,
        started: String,
        command: String,
    }

    fn write_fixture(root: &Path) {
        for name in ["Alpha", "Beta"] {
            let folder: std::path::PathBuf = root.join(name);
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(
                folder.join(format!("{name}.csproj")),
                r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>"#,
            )
            .unwrap();
            std::fs::write(
                folder.join("Code.cs"),
                format!(
                    "namespace {name};\npublic static class Code {{ public static int One() => 1; }}\n"
                ),
            )
            .unwrap();
        }
        std::fs::write(
            root.join("App.slnx"),
            r#"<Solution><Project Path="Alpha/Alpha.csproj" /><Project Path="Beta/Beta.csproj" /></Solution>"#,
        )
        .unwrap();
    }

    fn processes() -> (Vec<Proc>, u32) {
        let child: std::process::Child = Command::new("ps")
            .args(["-Ao", "pid=,pgid=,lstart=,command="])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let ps: u32 = child.id();
        let output: Output = child.wait_with_output().unwrap();
        let listed: Vec<Proc> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line: &str| {
                let words: Vec<&str> = line.split_whitespace().collect();
                Some(Proc {
                    pid: words.first()?.parse().ok()?,
                    group: words.get(1)?.parse().ok()?,
                    started: words.get(2..7)?.join(" "),
                    command: words.get(7..)?.join(" "),
                })
            })
            .collect();
        (listed, ps)
    }

    fn group_of(pid: u32, listed: &[Proc]) -> u32 {
        listed.iter().find(|p: &&Proc| p.pid == pid).unwrap().group
    }

    #[test]
    fn a_suite_build_leaves_no_compiler_server_or_node_and_uses_none() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        write_fixture(dir.path());
        let log: std::path::PathBuf = dir.path().join("roslyn.log");
        let (before, _): (Vec<Proc>, u32) = processes();
        let own_group: u32 = group_of(std::process::id(), &before);
        let ran: Ran = no_servers::run(
            no_servers::command()
                .current_dir(dir.path())
                .args(SUITE_BUILD)
                .env("RoslynCommandLineLogFile", &log),
        )
        .unwrap();
        assert!(ran.output.status.success(), "{:?}", ran.output);
        let (after, ps): (Vec<Proc>, u32) = processes();
        let left: Vec<Proc> = after
            .into_iter()
            .filter(|p: &Proc| !before.contains(p) && p.pid != ps && p.pid != std::process::id())
            .filter(|p: &Proc| p.group == ran.pid || p.group == own_group)
            .collect();
        let compiles: String = std::fs::read_to_string(&log).unwrap();
        let served: Vec<&str> = compiles
            .lines()
            .filter(|l: &&str| l.contains("CompilerServer:") || l.contains("named pipe"))
            .collect();
        let by_design: usize = served
            .iter()
            .filter(|l: &&&str| l.contains("by design"))
            .count();
        assert!(
            left.is_empty() && by_design == 2 && served.len() == 2,
            "started by the build and still alive: {left:#?}\ncompiler lines: {served:#?}"
        );
    }

    const LEAVES_A_STEP: &str = r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup><Target Name="LeaveAStep" AfterTargets="Build"><Exec Command="/bin/sleep 600 &gt;/dev/null 2&gt;&amp;1 &amp; echo $! &gt; step.pid" /></Target></Project>"#;

    fn alive(pid: &str) -> bool {
        Command::new("/bin/kill")
            .args(["-0", pid])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    }

    #[test]
    fn a_build_step_left_running_ends_with_the_build_group() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Step.csproj"), LEAVES_A_STEP).unwrap();
        std::fs::write(
            dir.path().join("Code.cs"),
            "namespace Step;
    public static class Code { }
    ",
        )
        .unwrap();
        let ran: Ran = no_servers::run(no_servers::command().current_dir(dir.path()).args([
            "build",
            "Step.csproj",
            "-c",
            "Debug",
            "-nologo",
            "-v",
            "q",
        ]))
        .unwrap();
        assert!(ran.output.status.success(), "{:?}", ran.output);
        let step: String = std::fs::read_to_string(dir.path().join("step.pid"))
            .unwrap()
            .trim()
            .to_owned();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let left: bool = alive(&step);
        if left {
            let _ended: std::io::Result<std::process::ExitStatus> =
                Command::new("/bin/kill").args(["-KILL", &step]).status();
        }
        assert!(!left, "the build's step {step} (sleep 600) outlived run");
    }

    const SPAWN: &str = concat!("Command::new(", "\"dotnet\")");
    const HELPER: &str = "bb_no_servers.rs";

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path: std::path::PathBuf = entry.unwrap().path();
            let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if path.is_dir() {
                rust_files(&path, out);
            } else if path
                .extension()
                .is_some_and(|e: &std::ffi::OsStr| e == "rs")
                && name != HELPER
            {
                out.push(path);
            }
        }
    }

    #[test]
    fn every_dotnet_the_workspace_starts_comes_from_the_helper() {
        let workspace: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        rust_files(&workspace.canonicalize().unwrap(), &mut files);
        assert!(files.len() > 100, "walked only {} files", files.len());
        let offenders: Vec<String> = files
            .iter()
            .filter(|path: &&std::path::PathBuf| {
                let text: String = std::fs::read_to_string(path).unwrap();
                let squeezed: String = text.split_whitespace().collect();
                squeezed.contains(SPAWN)
            })
            .map(|path: &std::path::PathBuf| path.display().to_string())
            .collect();
        assert!(
            offenders.is_empty(),
            "start dotnet with bb_dotnet::no_servers::command (no compiler server or node left): {offenders:#?}"
        );
    }
}
