//! A failed Apex list, explained.

use crate::TestkitError;

/// What a failed list most often means: an Apex CLI without IDE mode, which
/// ignores `--list --json` and starts a plain `dotnet test` (BB-BUG-061).
pub(super) const IDE_HINT: &str = "the Apex CLI must list with `test --list --json` \
(IDE mode, Apex.Cli 0.8.2-pre or later); an older one ignores those flags. \
Set apexTestkit.command to one with IDE mode, or remove the apexTestkit.* \
keys to list and run with dotnet test";

/// `error` with [`IDE_HINT`] after its detail when the list exited non-zero.
pub(super) fn with_ide_hint(error: TestkitError) -> TestkitError {
    if let TestkitError::Failed { what, code, detail } = error {
        TestkitError::Failed {
            what,
            code,
            detail: format!("{detail}\n{IDE_HINT}"),
        }
    } else {
        error
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bb_dotnet::ProjectPath;

    use super::IDE_HINT;
    use crate::backend::{ApexBackend, TempDir, Workspace};
    use crate::bb_engine::TestBackend;
    use crate::model::SpecList;
    use crate::parse::{ApexSettings, CommandLine};
    use crate::TestkitError;

    /// An Apex CLI from before IDE mode: it ignores `--list --json` and
    /// runs `dotnet test` on the project it is given, which fails.
    fn old_cli() -> CommandLine {
        CommandLine {
            program: "sh".to_owned(),
            prefix: vec![
                "-c".to_owned(),
                "echo \"dotnet test $3 -c Debug --no-build\"; \
                 echo 'error MSB4057: The target \"VSTest\" does not exist in the project.'; \
                 exit 1"
                    .to_owned(),
                "apex".to_owned(),
            ],
        }
    }

    /// A project whose `TargetPath` is a built file, so listing needs no build.
    fn built_project(temp: &TempDir) -> ProjectPath {
        let bin: PathBuf = temp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("Specs.dll"), "not a real assembly").unwrap();
        let project: PathBuf = temp.path().join("Specs.csproj");
        let text: String = format!(
            "<Project><PropertyGroup><TargetPath>{}</TargetPath></PropertyGroup></Project>",
            bin.join("Specs.dll").display()
        );
        std::fs::write(&project, text).unwrap();
        ProjectPath::from_path(project)
    }

    #[tokio::test]
    async fn a_list_by_an_apex_cli_without_ide_mode_says_how_to_fix_it() {
        let temp: TempDir = TempDir::new().unwrap();
        let project: ProjectPath = built_project(&temp);
        let backend: ApexBackend = ApexBackend::new(
            Workspace::at(temp.path().to_path_buf()),
            ApexSettings::defaults(),
        )
        .unwrap()
        .with_command(old_cli());
        let listed: Result<SpecList, TestkitError> = backend.list(&project).await;
        let detail: String = match listed {
            Err(TestkitError::Failed { detail, .. }) => detail,
            other => panic!("{other:?}"),
        };
        assert!(detail.contains("MSB4057"), "{detail}");
        assert!(detail.ends_with(IDE_HINT), "{detail}");
    }
}
