//! `bb audit` and `bb doctor` exit codes and bad-argument handling, end to end.

#[cfg(test)]
mod common;

use std::path::PathBuf;

use common::{bb, harness, Run};

#[test]
fn audit_of_the_harness_is_clean() {
    let root: PathBuf = harness();
    let run: Run = bb(&["audit", root.to_str().unwrap()]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    assert!(run.out.contains("audit     : clean"), "{}", run.out);
}

#[test]
fn audit_of_csharp_var_exits_non_zero() {
    let dir: PathBuf = std::env::temp_dir().join(format!("bb-e2e-var-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("App")).unwrap();
    std::fs::write(
        dir.join("App/App.cs"),
        "public class A\n{\n    void M()\n    {\n        var x = 1;\n    }\n}\n",
    )
    .unwrap();
    let run: Run = bb(&["audit", dir.to_str().unwrap()]);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&dir);
    assert_eq!(run.code, Some(1), "{}", run.all());
    assert!(run.out.contains("App/App.cs:5 CsVar"), "{}", run.out);
    assert!(run.err.contains("1 convention violations"), "{}", run.err);
}

#[test]
fn bad_args_exit_2_with_usage() {
    let sln: PathBuf = harness().join("Harness.slnx");
    let sln: &str = sln.to_str().unwrap();
    for args in [
        &[][..],
        &["fly"][..],
        &["scan"][..],
        &["open"][..],
        &["outline"][..],
        &["doctor", "extra"][..],
        &["scan", sln, "extra"][..],
        &["audit", ".", "extra"][..],
    ] {
        let run: Run = bb(args);
        assert_eq!(run.code, Some(2), "bb {args:?}: {}", run.all());
        assert!(run.err.contains("usage:"), "bb {args:?}: {}", run.err);
        assert!(run.out.is_empty(), "bb {args:?} ran: {}", run.out);
    }
}

#[cfg(test)]
mod doctor_env {
    use std::process::{Command, Output};

    /// `bb doctor` with `set` added to and `unset` removed from its environment.
    pub fn doctor(set: &[(&str, &str)], unset: &[&str]) -> (Option<i32>, String) {
        let mut cmd: Command = Command::new(env!("CARGO_BIN_EXE_bb"));
        cmd.arg("doctor");
        for (k, v) in set {
            cmd.env(k, v);
        }
        for k in unset {
            cmd.env_remove(k);
        }
        let out: Output = cmd.output().unwrap();
        let text: String = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code(), text)
    }
}

#[test]
fn doctor_fails_when_roslyn_is_missing() {
    let (code, text): (Option<i32>, String) =
        doctor_env::doctor(&[("BACKBACK_ROSLYN_LS", "/nonexistent/roslyn")], &[]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("roslyn-ls : MISSING"), "{text}");
}

#[test]
fn doctor_fails_when_dotnet_is_missing() {
    let (code, text): (Option<i32>, String) =
        doctor_env::doctor(&[("PATH", "/nonexistent")], &["DOTNET_ROOT"]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("dotnet    : MISSING"), "{text}");
}

#[test]
fn doctor_passes_without_netcoredbg_and_marks_it_optional() {
    let (code, text): (Option<i32>, String) =
        doctor_env::doctor(&[("BACKBACK_NETCOREDBG", "/nonexistent/netcoredbg")], &[]);
    assert_eq!(code, Some(0), "{text}");
    assert!(
        text.contains("netcoredbg: OPTIONAL (needed from BBEOD05)"),
        "{text}"
    );
}
