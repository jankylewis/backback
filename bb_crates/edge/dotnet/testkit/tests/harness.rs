//! The engine against real tools on a built copy of `fixtures/_Harness`:
//! the pinned Apex CLI, `dotnet vstest`/`dotnet test`, and an MTP app.

#[cfg(test)]
mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_dotnet::ProjectPath;
use bb_testkit::{
    run, ApexBackend, Category, Configuration, DebugHost, DotnetBackend, Fqn, Leaf, Outcome,
    RunReport, Selection, SpecList, TestBackend, TestkitError,
};
use common::{apex, build_copy, built, copy, processes_with, project};

const NUNIT: &str = "_NUnitApex/_NUnitApex.csproj";
const OUTCOME: &str = "Harness.NUnitApex.OutcomeSpec";

#[cfg(test)]
mod helpers {
    use super::*;

    pub fn names(list: &SpecList) -> BTreeSet<String> {
        list.specs
            .iter()
            .map(|f: &Fqn| f.as_str().to_owned())
            .collect()
    }

    pub fn specs(short: &[&str]) -> Vec<Fqn> {
        short
            .iter()
            .map(|s: &&str| Fqn::new(&format!("{OUTCOME}.{s}")).unwrap())
            .collect()
    }

    pub fn outcome_of(report: &RunReport, short: &str) -> Option<Outcome> {
        report.leaves.iter().find_map(|leaf: &Leaf| match leaf {
            Leaf::Ran(r) if r.fqn.as_str() == format!("{OUTCOME}.{short}") => Some(r.outcome),
            Leaf::Ran(_) | Leaf::Missing(_) => None,
        })
    }

    pub fn vstest(root: &Path) -> DotnetBackend {
        DotnetBackend::for_project(&project(root, NUNIT), Configuration::debug(), None, false)
            .unwrap()
    }
}

use helpers::{names, outcome_of, specs, vstest};

#[tokio::test]
async fn apex_and_vstest_list_the_same_six_specs() {
    let root: &Path = built();
    let by_apex: SpecList = apex(root).list(&project(root, NUNIT)).await.unwrap();
    let by_vstest: SpecList = vstest(root).list(&project(root, NUNIT)).await.unwrap();
    assert_eq!(by_apex.project, "_NUnitApex");
    assert_eq!(by_apex.specs.len(), 6, "{by_apex:?}");
    assert_eq!(names(&by_apex), names(&by_vstest));
    assert!(names(&by_apex).contains("Harness.NUnitApex.GreeterSpec.Greets(\"\",\"hi there\")"));
}

#[tokio::test]
async fn an_apex_run_maps_every_outcome_and_errors_a_spec_absent_from_trx() {
    let root: &Path = built();
    let selection: Selection = Selection {
        specs: specs(&["Passes", "Fails", "Skipped", "Undecided", "NoSuchSpec"]),
        category: None,
    };
    let report: RunReport = run(&apex(root), &project(root, NUNIT), &selection)
        .await
        .unwrap();
    assert_eq!(outcome_of(&report, "Passes"), Some(Outcome::Passed));
    assert_eq!(outcome_of(&report, "Fails"), Some(Outcome::Failed));
    assert_eq!(outcome_of(&report, "Skipped"), Some(Outcome::NotExecuted));
    assert_eq!(outcome_of(&report, "Undecided"), Some(Outcome::NotExecuted));
    assert_eq!(
        report.leaves.last(),
        Some(&Leaf::Missing(specs(&["NoSuchSpec"]).remove(0)))
    );
    assert_eq!(report.counts(), (1, 1, 2, 1));
    assert_ne!(report.exit_code, Some(0));
}

#[tokio::test]
async fn a_vstest_run_filters_parameterised_names_and_categories() {
    let root: &Path = built();
    let greets: Fqn = Fqn::new("Harness.NUnitApex.GreeterSpec.Greets(\"\",\"hi there\")").unwrap();
    let by_name: Selection = Selection {
        specs: vec![greets.clone()],
        category: None,
    };
    let named: RunReport = run(&vstest(root), &project(root, NUNIT), &by_name)
        .await
        .unwrap();
    assert_eq!(named.leaves.len(), 1, "{named:?}");
    assert!(
        matches!(&named.leaves[0], Leaf::Ran(r) if r.fqn == greets && r.outcome == Outcome::Passed)
    );
    let smoke: Selection = Selection {
        specs: Vec::new(),
        category: Category::new("Smoke"),
    };
    let tagged: RunReport = run(&vstest(root), &project(root, NUNIT), &smoke)
        .await
        .unwrap();
    assert_eq!(tagged.counts(), (1, 0, 0, 0), "{tagged:?}");
    assert_eq!(outcome_of(&tagged, "Passes"), Some(Outcome::Passed));
}

#[tokio::test]
async fn an_mtp_app_lists_and_runs_with_a_trx_result() {
    let root: &Path = built();
    let app: ProjectPath = project(root, "XunitPlaywright/_XunitPlaywright.csproj");
    let backend: DotnetBackend =
        DotnetBackend::for_project(&app, Configuration::debug(), None, false).unwrap();
    assert_eq!(backend.name(), "mtp");
    let list: SpecList = backend.list(&app).await.unwrap();
    assert_eq!(
        names(&list),
        BTreeSet::from(["Harness.XunitPlaywright.HomePageSpec.TitleIsShown".to_owned()])
    );
    let selection: Selection = Selection {
        specs: list.specs.clone(),
        category: None,
    };
    let report: RunReport = run(&backend, &app, &selection).await.unwrap();
    assert_eq!(report.leaves.len(), 1, "{report:?}");
    assert!(
        matches!(&report.leaves[0], Leaf::Ran(r) if r.duration > Duration::ZERO),
        "{report:?}"
    );
}

#[tokio::test]
async fn a_run_that_writes_no_trx_is_an_error_even_with_exit_zero() {
    let root: &Path = built();
    let silent: ApexBackend =
        apex(root).with_command(bb_testkit::CommandLine::parse("true").unwrap());
    let outcome: Result<RunReport, TestkitError> = run(
        &silent,
        &project(root, NUNIT),
        &Selection {
            specs: specs(&["Passes"]),
            category: None,
        },
    )
    .await;
    assert!(
        matches!(outcome, Err(TestkitError::NoTrx(_))),
        "{outcome:?}"
    );
}

#[tokio::test]
async fn an_unbuilt_project_and_a_category_on_apex_are_typed_errors() {
    let root: PathBuf = copy("unbuilt");
    let listed: Result<SpecList, TestkitError> = vstest(&root).list(&project(&root, NUNIT)).await;
    let by_category: Result<RunReport, TestkitError> = run(
        &apex(&root),
        &project(&root, NUNIT),
        &Selection {
            specs: specs(&["Passes"]),
            category: Category::new("Smoke"),
        },
    )
    .await;
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&root);
    assert!(
        matches!(listed, Err(TestkitError::NotBuilt(_))),
        "{listed:?}"
    );
    assert!(
        matches!(
            by_category,
            Err(TestkitError::Unsupported {
                backend: "apex",
                ..
            })
        ),
        "{by_category:?}"
    );
}

#[tokio::test]
async fn a_backend_that_builds_first_runs_an_unbuilt_project() {
    let root: PathBuf = copy("buildfirst");
    let backend: DotnetBackend =
        DotnetBackend::for_project(&project(&root, NUNIT), Configuration::debug(), None, true)
            .unwrap();
    let selection: Selection = Selection {
        specs: specs(&["Passes"]),
        category: None,
    };
    let report: Result<RunReport, TestkitError> =
        run(&backend, &project(&root, NUNIT), &selection).await;
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&root);
    assert_eq!(report.unwrap().counts(), (1, 0, 0, 0));
}

#[tokio::test]
async fn an_apex_debug_run_gives_the_waiting_host_and_stop_leaves_no_process_behind() {
    let root: PathBuf = build_copy("debug", &[NUNIT]);
    let trx: PathBuf = root.join("debug.trx");
    let host: DebugHost = apex(&root)
        .debug(
            &project(&root, NUNIT),
            &specs(&["Passes"]),
            &trx,
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let pid: String = host.pid.get().to_string();
    let waiting: Vec<String> = processes_with(&format!("{}/_NUnitApex/bin", root.display()));
    host.stop().await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let left: Vec<String> = processes_with(root.to_str().unwrap());
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&root);
    assert!(
        waiting
            .iter()
            .any(|p: &String| p.starts_with(&format!("{pid} ")) && p.contains("testhost")),
        "{pid} is not a waiting test host: {waiting:?}"
    );
    assert!(left.is_empty(), "left running after stop: {left:?}");
}

#[tokio::test]
async fn a_list_whose_test_host_crashes_is_an_error_not_zero_specs() {
    let root: PathBuf = build_copy("hostcrash", &[NUNIT]);
    let bin: PathBuf = root.join("_NUnitApex/bin/Debug/net10.0");
    std::fs::remove_file(bin.join("NUnitApex.runtimeconfig.json")).unwrap();
    std::fs::remove_file(bin.join("NUnitApex.deps.json")).unwrap();
    let by_apex: Result<SpecList, TestkitError> = apex(&root).list(&project(&root, NUNIT)).await;
    let by_vstest: Result<SpecList, TestkitError> =
        vstest(&root).list(&project(&root, NUNIT)).await;
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(&root);
    for listed in [by_apex, by_vstest] {
        assert!(
            matches!(&listed, Err(TestkitError::Failed { detail, .. }) if detail.contains("Testhost")),
            "{listed:?}"
        );
    }
}
