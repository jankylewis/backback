//! Tests for project detection and runsettings, on throwaway trees.

use super::*;

/// A unique temp folder per test, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("bb-dotnet-{tag}-{}", std::process::id()));
        let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let path: PathBuf = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&self.0);
    }
}

fn project(path: PathBuf) -> ProjectPath {
    ProjectPath::from_path(path)
}

#[test]
fn nunit_apex_project_with_runsettings_is_recognised() {
    let s: Scratch = Scratch::new("apex");
    let csproj: PathBuf = s.write(
        "MbDesktop/MbDesktop.csproj",
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><OutputType>Exe</OutputType><IsTestProject>true</IsTestProject>
    <RunSettingsFilePath>$(MSBuildProjectDirectory)/apextest.runsettings</RunSettingsFilePath></PropertyGroup>
  <ItemGroup><PackageReference Include="Apex" /><PackageReference Include="NUnit" />
    <PackageReference Include="NUnit3TestAdapter" /></ItemGroup>
</Project>"#,
    );
    s.write(
        "MbDesktop/apextest.runsettings",
        "<RunSettings><RunConfiguration><EnvironmentVariables>\
         <MBING_HEADLESS>true</MBING_HEADLESS><MODEC_PASSWORD>hunter2</MODEC_PASSWORD>\
         <MODEC_TOTP_KEY>__FILL_ME__</MODEC_TOTP_KEY><APEX_PATH>/opt</APEX_PATH>\
         </EnvironmentVariables></RunConfiguration><NUnit><NumberOfTestWorkers>2</NumberOfTestWorkers></NUnit></RunSettings>",
    );
    let info: ProjectInfo = ProjectInfo::load(&project(csproj)).unwrap();
    assert!(info.is_test && info.is_exe);
    assert_eq!(info.capabilities, vec![Capability::NUnit, Capability::Apex]);
    let rs: RunSettings = info.runsettings.unwrap();
    assert_eq!(rs.nunit_workers, Some(2));
    let values: Vec<(&str, &EnvValue)> = rs
        .env
        .iter()
        .map(|v: &EnvVar| (v.name.as_str(), &v.value))
        .collect();
    assert_eq!(
        values,
        vec![
            ("MBING_HEADLESS", &EnvValue::Plain("true".to_owned())),
            ("MODEC_PASSWORD", &EnvValue::Secret),
            ("MODEC_TOTP_KEY", &EnvValue::Unset),
            ("APEX_PATH", &EnvValue::Plain("/opt".to_owned())),
        ]
    );
    assert_eq!(rs.unset(), vec!["MODEC_TOTP_KEY"]);
}

#[test]
fn secret_text_is_never_kept() {
    let v: EnvVar = classify("ADO_PAT", "abc123secret");
    assert_eq!(v.value, EnvValue::Secret);
    assert!(!format!("{v:?}").contains("abc123secret"));
}

#[test]
fn playwright_xunit_apex_and_props_inheritance() {
    let s: Scratch = Scratch::new("pw");
    s.write(
        "Directory.Build.props",
        "<Project><PropertyGroup><UseMicrosoftTestingPlatformRunner>true</UseMicrosoftTestingPlatformRunner></PropertyGroup></Project>",
    );
    let csproj: PathBuf = s.write(
        "src/Ui/Ui.csproj",
        r#"<Project><ItemGroup><PackageReference Include="Microsoft.Playwright.Xunit" />
  <PackageReference Include="xunit.v3" /><PackageReference Include="ModelContextProtocol" />
  <ProjectReference Include="..\..\Apex\src\Apex\Apex.csproj" /></ItemGroup></Project>"#,
    );
    let info: ProjectInfo = ProjectInfo::load(&project(csproj)).unwrap();
    assert!(info.is_test && !info.is_exe && info.runsettings.is_none());
    assert_eq!(
        info.capabilities,
        vec![
            Capability::XUnit,
            Capability::Playwright,
            Capability::Apex,
            Capability::TestingPlatform
        ]
    );
}

#[test]
fn library_without_tests_is_not_a_test_project() {
    let s: Scratch = Scratch::new("lib");
    let csproj: PathBuf = s.write(
        "Core/Core.csproj",
        "<Project><ItemGroup><PackageReference Include=\"Apexium\" /></ItemGroup></Project>",
    );
    let info: ProjectInfo = ProjectInfo::load(&project(csproj)).unwrap();
    assert!(!info.is_test);
    assert!(
        info.capabilities.is_empty(),
        "Apexium is not Apex: {:?}",
        info.capabilities
    );
}
