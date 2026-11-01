//! Native window regression for test discovery across the process boundary.

use std::path::Path;
use std::process::{Command, ExitStatus, Output};

fn fixture(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        root.join("Tests.csproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1" /><PackageReference Include="NUnit" Version="4.6.1" /><PackageReference Include="NUnit3TestAdapter" Version="5.0.0" /></ItemGroup></Project>"#,
    )?;
    std::fs::write(root.join("OutcomeSpec.cs"), "using NUnit.Framework;\nnamespace Sample;\npublic class OutcomeSpec {\n[Test] public void Passes() => Assert.Pass();\n[Test] public void Fails() => Assert.Fail(\"failure\");\n}\n")?;
    let status: ExitStatus = Command::new("dotnet")
        .args(["build", "Tests.csproj", "-c", "Debug", "-nologo", "-v", "q"])
        .current_dir(root)
        .status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn native_window_discovers_tests_after_expanding_project() -> Result<(), Box<dyn std::error::Error>>
{
    let dir: tempfile::TempDir = tempfile::tempdir()?;
    fixture(dir.path())?;
    let output: Output = Command::new(env!("CARGO_BIN_EXE_backback"))
        .arg(dir.path().join("OutcomeSpec.cs"))
        .env("BB_KEYS", "tests sleep:1000 tests:expand sleep:6000 state")
        .env("BB_KEYS_QUIT", "1")
        .output()?;
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    assert!(
        stderr.contains("bb_tests_state projects=1 discovered=2 loading=0"),
        "{stderr}"
    );
    Ok(())
}
