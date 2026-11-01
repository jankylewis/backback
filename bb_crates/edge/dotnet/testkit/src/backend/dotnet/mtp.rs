//! Microsoft.Testing.Platform: the test project is its own app. List with
//! `dotnet exec <app> --list-tests`, run with filters and a TRX report.
//! (`dotnet test` without `global.json` opting in runs MTP apps through
//! VSTest, which fails to start them.)

use std::path::{Path, PathBuf};

use bb_dotnet::ProjectPath;

use super::DotnetBackend;
use crate::backend::{project_name, shadow, Call, Finished, TempDir};
use crate::bb_engine::Selection;
use crate::model::{Fqn, SpecList};
use crate::parse::parse_mtp_list;
use crate::TestkitError;

pub(super) async fn list(
    backend: &DotnetBackend,
    project: &ProjectPath,
    assembly: PathBuf,
) -> Result<SpecList, TestkitError> {
    let folder: TempDir = TempDir::new()?;
    let safe_assembly: PathBuf = shadow::assembly(&assembly, &folder)?;
    let done: Finished = app(backend, &safe_assembly)
        .args(["--list-tests"])
        .discover(backend.deadlines.list)
        .await?;
    Ok(SpecList {
        project: project_name(project),
        specs: parse_mtp_list(&done.stdout)?,
        assembly,
        elapsed: done.elapsed,
    })
}

pub(super) async fn execute(
    backend: &DotnetBackend,
    assembly: &Path,
    selection: &Selection,
    trx: &Path,
) -> Result<Finished, TestkitError> {
    let folder: &Path = trx.parent().unwrap_or_else(|| Path::new("."));
    let file: String = trx
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (report, report_name): (&str, &str) = if backend.xunit {
        ("--report-xunit-trx", "--report-xunit-trx-filename")
    } else {
        ("--report-trx", "--report-trx-filename")
    };
    app(backend, assembly)
        .args(filters(selection))
        .args(["--results-directory".into(), folder.as_os_str().to_owned()])
        .args([report.to_owned(), report_name.to_owned(), file])
        .finish(backend.deadlines.run)
        .await
}

/// `--filter-method` per spec (method level: arguments dropped) and
/// `--filter-trait Category=X`.
#[must_use]
pub(crate) fn filters(selection: &Selection) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut methods: Vec<&str> = selection.specs.iter().map(Fqn::method).collect();
    methods.dedup();
    for method in methods {
        args.push("--filter-method".to_owned());
        args.push(method.to_owned());
    }
    if let Some(category) = &selection.category {
        args.push("--filter-trait".to_owned());
        args.push(format!("Category={}", category.as_str()));
    }
    args
}

fn app(backend: &DotnetBackend, assembly: &Path) -> Call {
    Call::new(backend.sdk.host(), backend.workspace.root())
        .args(["exec".into(), assembly.as_os_str().to_owned()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    #[test]
    fn filters_name_each_method_once_and_the_category_trait() {
        let selection: Selection = Selection {
            specs: ["N.C.M(1)", "N.C.M(2)", "N.C.K"]
                .iter()
                .map(|s: &&str| Fqn::new(s).unwrap())
                .collect(),
            category: Category::new("Smoke"),
        };
        assert_eq!(
            filters(&selection),
            [
                "--filter-method",
                "N.C.M",
                "--filter-method",
                "N.C.K",
                "--filter-trait",
                "Category=Smoke"
            ]
        );
    }
}
