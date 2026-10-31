//! `bb audit` exit codes and bad-argument handling, end to end.

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
