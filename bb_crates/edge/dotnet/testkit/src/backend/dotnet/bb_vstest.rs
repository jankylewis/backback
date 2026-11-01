//! VSTest: list with `dotnet vstest --ListFullyQualifiedTests`, run with
//! `dotnet test --filter ... --logger trx`. (`dotnet test --list-tests`
//! prints display names only, without namespace and class.)

use std::path::{Path, PathBuf};

use bb_dotnet::ProjectPath;

use super::DotnetBackend;
use crate::backend::{project_name, shadow, Call, Finished, TempDir};
use crate::bb_engine::Selection;
use crate::model::{Fqn, SpecList};
use crate::parse::parse_fqn_lines;
use crate::TestkitError;

/// Characters VSTest filter values escape with `\`.
const FILTER_SPECIAL: &str = "\\()|&=!~";

pub(super) async fn list(
    backend: &DotnetBackend,
    project: &ProjectPath,
    assembly: PathBuf,
) -> Result<SpecList, TestkitError> {
    let folder: TempDir = TempDir::new()?;
    let safe_assembly: PathBuf = shadow::assembly(&assembly, &folder)?;
    let target: PathBuf = folder.path().join("tests.txt");
    let mut call: Call = Call::new(backend.sdk.host(), folder.path())
        .args(["vstest".into(), safe_assembly.as_os_str().to_owned()])
        .args([
            "--ListFullyQualifiedTests".to_owned(),
            format!("--ListTestsTargetPath:{}", target.display()),
        ]);
    if let Some(flag) = &backend.stop {
        call = call.stoppable(flag.clone());
    }
    let done: Finished = call.discover(backend.deadlines.list).await?;
    let text: String =
        std::fs::read_to_string(&target).map_err(|source: std::io::Error| TestkitError::Io {
            path: target.clone(),
            source,
        })?;
    Ok(SpecList {
        project: project_name(project),
        assembly,
        specs: parse_fqn_lines(&text)?,
        elapsed: done.elapsed,
    })
}

pub(super) async fn execute(
    backend: &DotnetBackend,
    project: &ProjectPath,
    selection: &Selection,
    trx: &Path,
) -> Result<Finished, TestkitError> {
    let mut call: Call = Call::new(backend.sdk.host(), backend.workspace.root())
        .args(["test".into(), project.as_path().as_os_str().to_owned()])
        .args([
            "-c",
            backend.configuration.as_str(),
            "--no-build",
            "--nologo",
        ]);
    if let Some(filter) = filter(selection) {
        call = call.args(["--filter".to_owned(), filter]);
    }
    if let Some(settings) = &backend.settings {
        call = call.args(["--settings".into(), settings.as_os_str().to_owned()]);
    }
    let call: Call = call.args([
        "--logger".to_owned(),
        format!("trx;LogFileName={}", trx.display()),
    ]);
    let call: Call = match &backend.stop {
        Some(stop) => call.stoppable(stop.clone()),
        None => call,
    };
    call.finish(backend.deadlines.run).await
}

/// The `--filter` for a selection, escaped for VSTest and for MSBuild
/// (`dotnet test` passes it through MSBuild, as the Apex CLI does).
#[must_use]
pub(crate) fn filter(selection: &Selection) -> Option<String> {
    let names: Vec<String> = selection
        .specs
        .iter()
        .map(|f: &Fqn| format!("FullyQualifiedName={}", escape(f.as_str())))
        .collect();
    let category: Option<String> = selection
        .category
        .as_ref()
        .map(|c: &crate::model::Category| format!("TestCategory={}", escape(c.as_str())));
    let joined: String = match (names.is_empty(), category) {
        (true, None) => return None,
        (true, Some(c)) => c,
        (false, None) => names.join("|"),
        (false, Some(c)) => format!("({})&{c}", names.join("|")),
    };
    Some(for_msbuild(&joined))
}

fn escape(value: &str) -> String {
    let mut out: String = String::with_capacity(value.len());
    for c in value.chars() {
        if FILTER_SPECIAL.contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn for_msbuild(filter: &str) -> String {
    filter
        .replace('%', "%25")
        .replace('"', "%22")
        .replace(',', "%2C")
        .replace(';', "%3B")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    fn pick(specs: &[&str], category: Option<&str>) -> Selection {
        Selection {
            specs: specs.iter().map(|s: &&str| Fqn::new(s).unwrap()).collect(),
            category: category.and_then(Category::new),
        }
    }

    #[test]
    fn filter_escapes_vstest_and_msbuild_characters_like_the_apex_cli() {
        let one: Selection = pick(&["N.C.Greets(\"a,b\";1)"], None);
        assert_eq!(
            filter(&one).unwrap(),
            "FullyQualifiedName=N.C.Greets\\(%22a%2Cb%22%3B1\\)"
        );
        let two: Selection = pick(&["N.C.A", "N.C.B"], Some("Smoke"));
        assert_eq!(
            filter(&two).unwrap(),
            "(FullyQualifiedName=N.C.A|FullyQualifiedName=N.C.B)&TestCategory=Smoke"
        );
        assert_eq!(
            filter(&pick(&[], Some("Smoke"))).unwrap(),
            "TestCategory=Smoke"
        );
        assert_eq!(filter(&pick(&[], None)), None);
    }
}
