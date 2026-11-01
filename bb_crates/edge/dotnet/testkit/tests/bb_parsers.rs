//! The readers on real captured output (`tests/data`) and on broken copies.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_testkit::{
    parse_apex_list, parse_host_pid, parse_mtp_list, parse_trx, ApexList, ApexSettings, Fqn,
    FqnError, HostPid, Outcome, OutputError, SpecResult, TrxError,
};

#[cfg(test)]
mod helpers {
    use super::*;

    pub fn data(name: &str) -> String {
        let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    pub fn by_name<'a>(results: &'a [SpecResult], name: &str) -> &'a SpecResult {
        results
            .iter()
            .find(|r: &&SpecResult| r.fqn.as_str() == name)
            .unwrap()
    }
}

use helpers::{by_name, data};

#[test]
fn apex_trx_gives_every_outcome_with_name_time_message_and_stack() {
    let results: Vec<SpecResult> = parse_trx(&data("bb_apex_run.trx")).unwrap();
    assert_eq!(results.len(), 5);
    let greets: &SpecResult = by_name(
        &results,
        "Harness.NUnitApex.GreeterSpec.Greets(\"Vinh\",\"hi Vinh\")",
    );
    assert_eq!(greets.outcome, Outcome::Passed);
    assert_eq!(greets.duration, Duration::from_micros(2706));
    let fails: &SpecResult = by_name(&results, "Harness.NUnitApex.OutcomeSpec.Fails");
    assert_eq!(fails.outcome, Outcome::Failed);
    assert!(fails
        .message
        .as_deref()
        .unwrap()
        .contains("two is not three"));
    assert!(fails
        .stack
        .as_deref()
        .unwrap()
        .contains("OutcomeSpec.Fails()"));
    let skipped: &SpecResult = by_name(&results, "Harness.NUnitApex.OutcomeSpec.Skipped");
    assert_eq!(skipped.outcome, Outcome::NotExecuted);
    assert_eq!(skipped.message.as_deref(), Some("skipped on purpose"));
    let passes: &SpecResult = by_name(&results, "Harness.NUnitApex.OutcomeSpec.Passes");
    assert_eq!((passes.message.clone(), passes.stack.clone()), (None, None));
}

#[test]
fn mtp_trx_names_the_test_by_class_and_method() {
    let results: Vec<SpecResult> = parse_trx(&data("mtp_run.trx")).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].fqn.as_str(),
        "Harness.XunitPlaywright.HomePageSpec.TitleIsShown"
    );
    assert_eq!(results[0].outcome, Outcome::Failed);
    assert_eq!(results[0].duration, Duration::from_nanos(1_948_809_800));
}

#[test]
fn inconclusive_is_an_outcome_and_an_unknown_outcome_is_an_error() {
    let trx: String = data("bb_apex_run.trx");
    let inconclusive: String = trx.replacen("outcome=\"Passed\"", "outcome=\"Inconclusive\"", 1);
    let outcomes: Vec<Outcome> = parse_trx(&inconclusive)
        .unwrap()
        .iter()
        .map(|r: &SpecResult| r.outcome)
        .collect();
    assert!(outcomes.contains(&Outcome::Inconclusive));
    let unknown: Result<Vec<SpecResult>, TrxError> =
        parse_trx(&trx.replacen("outcome=\"Passed\"", "outcome=\"Timeout\"", 1));
    assert!(
        matches!(&unknown, Err(TrxError::UnknownOutcome { value, .. }) if value == "Timeout"),
        "{unknown:?}"
    );
}

#[test]
fn a_trx_without_results_or_definitions_or_xml_is_refused() {
    let trx: String = data("bb_apex_run.trx");
    let no_results: String = trx
        .replace("<Results>", "<Gone>")
        .replace("</Results>", "</Gone>");
    assert_eq!(parse_trx(&no_results), Err(TrxError::NoResults));
    assert_eq!(parse_trx("<Other/>"), Err(TrxError::NoResults));
    let no_defs: String = trx
        .replace("<TestDefinitions>", "<Gone>")
        .replace("</TestDefinitions>", "</Gone>");
    assert!(matches!(parse_trx(&no_defs), Err(TrxError::UnknownTest(_))));
    assert!(matches!(
        parse_trx(&trx[..trx.len() / 2]),
        Err(TrxError::Xml(_))
    ));
    let bad_time: String = trx.replacen("duration=\"00:00:00.0027060\"", "duration=\"0:0:0\"", 1);
    assert_eq!(
        parse_trx(&bad_time),
        Err(TrxError::BadDuration("0:0:0".to_owned()))
    );
}

#[test]
fn apex_list_json_gives_the_specs_and_refuses_any_other_shape() {
    let list: ApexList = parse_apex_list(&data("apex_list.json")).unwrap();
    assert_eq!(list.project, "_NUnitApex");
    assert!(list.assembly.ends_with("NUnitApex.dll"));
    assert_eq!(list.specs.len(), 6);
    assert_eq!(
        list.specs[1].as_str(),
        "Harness.NUnitApex.GreeterSpec.Greets(\"\",\"hi there\")"
    );
    let extra: String =
        data("apex_list.json").replacen("{\"project\"", "{\"version\":2,\"project\"", 1);
    assert!(matches!(parse_apex_list(&extra), Err(OutputError::Json(_))));
    let noise: String = format!("Building...\n{}", data("apex_list.json"));
    assert!(matches!(parse_apex_list(&noise), Err(OutputError::Json(_))));
    let nameless: &str = r#"{"project":"p","assembly":"a","specs":[{"fqn":"NoDots"}]}"#;
    assert!(matches!(
        parse_apex_list(nameless),
        Err(OutputError::Name(FqnError::NoClass(_)))
    ));
}

#[test]
fn mtp_list_keeps_the_lines_after_the_header() {
    let out: &str = "xUnit.net v3 Microsoft.Testing.Platform Runner v2.0.2\n\nThe following Tests are available:\nHarness.X.HomePageSpec.TitleIsShown\n";
    let specs: Vec<Fqn> = parse_mtp_list(out).unwrap();
    assert_eq!(
        specs,
        vec![Fqn::new("Harness.X.HomePageSpec.TitleIsShown").unwrap()]
    );
    assert_eq!(
        parse_mtp_list("no header\nA.B.C\n").unwrap(),
        Vec::<Fqn>::new()
    );
}

#[test]
fn settings_read_jsonc_resolve_paths_and_default_absent_keys() {
    let root: &Path = Path::new("/ws");
    let found: ApexSettings = ApexSettings::parse(&data("settings.jsonc"), root).unwrap();
    assert_eq!(found.command.program, "dotnet");
    assert_eq!(found.command.prefix, vec!["apex".to_owned()]);
    assert_eq!(found.configuration.as_str(), "Debug");
    assert_eq!(
        found.run_settings,
        Some(root.join("MbAuto/RgvMbDesktop/mbingdt_apextest.runsettings"))
    );
    assert!(!found.build_before_run && found.declared);
    assert_eq!(found.solution, None);
    let empty: ApexSettings = ApexSettings::parse("{ // nothing\n}", root).unwrap();
    assert_eq!(empty, ApexSettings::defaults());
    assert!(empty.build_before_run);
    assert!(ApexSettings::parse(r#"{"apexTestkit.buildBeforeRun": "no"}"#, root).is_err());
    assert!(ApexSettings::parse(r#"{"apexTestkit.command": "   "}"#, root).is_err());
}

#[test]
fn host_pid_is_read_from_the_apex_line_and_the_vstest_line() {
    assert_eq!(
        parse_host_pid("{\"testHostPid\":4242}").map(HostPid::get),
        Some(4242)
    );
    assert_eq!(
        parse_host_pid("Host debugging is enabled. Process Id: 77 Name: testhost")
            .map(HostPid::get),
        Some(77)
    );
    for bad in [
        "{\"testHostPid\":0}",
        "{\"testHostPid\":-1}",
        "{\"other\":1}",
        "Process Id: x",
        "hello",
    ] {
        assert_eq!(parse_host_pid(bad), None, "{bad}");
    }
}

#[test]
fn a_name_splits_into_namespace_class_and_spec_with_dots_in_arguments() {
    let name: Fqn = Fqn::new("A.B.Outer+Inner.Runs(\"a.b\", 1.5)").unwrap();
    let parts: bb_testkit::FqnParts<'_> = name.parts();
    assert_eq!(
        (parts.namespace, parts.class, parts.spec),
        ("A.B", "Outer+Inner", "Runs(\"a.b\", 1.5)")
    );
    assert_eq!(name.method(), "A.B.Outer+Inner.Runs");
    let global: Fqn = Fqn::new("C.M").unwrap();
    assert_eq!((global.parts().namespace, global.parts().class), ("", "C"));
    assert_eq!(Fqn::new("  "), Err(FqnError::Empty));
    assert!(matches!(Fqn::new("M(a.b)"), Err(FqnError::NoClass(_))));
    assert!(matches!(Fqn::new(".M"), Err(FqnError::NoClass(_))));
}
