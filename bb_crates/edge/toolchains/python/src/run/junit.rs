//! pytest's names and results: a node id (`pkg/test_m.py::Cls::test_x[1]`)
//! is the Tests panel's `pkg.test_m.Cls.test_x(1)`, the same name pytest's
//! junit XML gives (`classname` + `.` + `name`); a junit report becomes
//! one result per test case.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_testkit::{Fqn, Outcome, SpecResult};

use crate::PythonError;

/// The panel's name for a pytest node id; `None` when it has no test part.
#[must_use]
pub fn fqn_of(node_id: &str) -> Option<Fqn> {
    let (file, rest): (&str, &str) = node_id.split_once("::")?;
    let module: String = file.strip_suffix(".py").unwrap_or(file).replace('/', ".");
    Fqn::new(&params_to_fqn(&format!(
        "{module}.{}",
        rest.replace("::", ".")
    )))
    .ok()
}

/// The pytest node id of `fqn` under `dir`: the longest leading parts that
/// name a `.py` file there are the module, the rest the test.
#[must_use]
pub fn node_id_of(dir: &Path, fqn: &Fqn) -> Option<String> {
    let text: &str = fqn.as_str();
    let (head, params): (&str, &str) = match text.split_once('(') {
        Some((head, args)) => (head, args.strip_suffix(')').unwrap_or(args)),
        None => (text, ""),
    };
    let parts: Vec<&str> = head.split('.').collect();
    let module_len: usize = (1..parts.len()).rev().find(|k: &usize| {
        let file: PathBuf = dir.join(format!(
            "{}.py",
            parts.get(..*k).unwrap_or_default().join("/")
        ));
        file.is_file()
    })?;
    let module: String = parts.get(..module_len)?.join("/");
    let case: String = parts.get(module_len..)?.join("::");
    let id: String = format!("{module}.py::{case}");
    Some(if params.is_empty() {
        id
    } else {
        format!("{id}[{params}]")
    })
}

/// `test_x[1-2]` becomes `test_x(1-2)`: the Tests panel keeps arguments
/// in parentheses, where their dots are not name parts.
fn params_to_fqn(name: &str) -> String {
    match name.split_once('[') {
        Some((head, args)) => format!("{head}({})", args.strip_suffix(']').unwrap_or(args)),
        None => name.to_owned(),
    }
}

/// One result per `testcase` of a junit XML report.
///
/// # Errors
/// [`PythonError::Unreadable`] when the text is not XML.
pub fn results(xml: &str) -> Result<Vec<SpecResult>, PythonError> {
    let doc: roxmltree::Document<'_> =
        roxmltree::Document::parse(xml).map_err(|error: roxmltree::Error| {
            PythonError::Unreadable {
                what: String::from("pytest junit report"),
                detail: error.to_string(),
            }
        })?;
    Ok(doc
        .descendants()
        .filter(|node: &roxmltree::Node<'_, '_>| node.has_tag_name("testcase"))
        .filter_map(|case: roxmltree::Node<'_, '_>| result(case))
        .collect())
}

fn result(case: roxmltree::Node<'_, '_>) -> Option<SpecResult> {
    let class: &str = case.attribute("classname").unwrap_or_default();
    let name: &str = case.attribute("name")?;
    let joined: String = if class.is_empty() {
        name.to_owned()
    } else {
        format!("{class}.{name}")
    };
    let fqn: Fqn = Fqn::new(&params_to_fqn(&joined)).ok()?;
    let seconds: f64 = case
        .attribute("time")
        .and_then(|t: &str| t.parse().ok())
        .unwrap_or_default();
    let verdict: Option<roxmltree::Node<'_, '_>> =
        case.children().find(|n: &roxmltree::Node<'_, '_>| {
            n.has_tag_name("failure") || n.has_tag_name("error") || n.has_tag_name("skipped")
        });
    let outcome: Outcome = match verdict.map(|n: roxmltree::Node<'_, '_>| n.tag_name().name()) {
        Some("failure" | "error") => Outcome::Failed,
        Some(_) => Outcome::NotExecuted,
        None => Outcome::Passed,
    };
    let message: Option<String> =
        verdict.and_then(|n: roxmltree::Node<'_, '_>| n.attribute("message").map(str::to_owned));
    let stack: Option<String> = verdict
        .filter(|_: &roxmltree::Node<'_, '_>| outcome == Outcome::Failed)
        .and_then(|n: roxmltree::Node<'_, '_>| n.text().map(str::to_owned));
    Some(SpecResult {
        fqn,
        outcome,
        duration: Duration::try_from_secs_f64(seconds).unwrap_or_default(),
        message,
        stack,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = r#"<testsuites><testsuite name="pytest">
<testcase classname="tests.test_calc" name="test_passes" time="0.001"/>
<testcase classname="tests.test_calc" name="test_fails" time="0.002"><failure message="assert 3 == 4">trace</failure></testcase>
<testcase classname="tests.test_calc" name="test_skipped" time="0"><skipped type="pytest.skip" message="not today">skip</skipped></testcase>
<testcase classname="tests.test_calc.TestBox" name="test_p[1.5-x]" time="0"/>
</testsuite></testsuites>"#;

    #[test]
    fn junit_cases_become_pass_fail_and_skip_with_their_messages() {
        let found: Vec<(String, Outcome, Option<String>)> = results(REPORT)
            .unwrap()
            .into_iter()
            .map(|r: SpecResult| (r.fqn.as_str().to_owned(), r.outcome, r.message))
            .collect();
        assert_eq!(
            found,
            vec![
                (
                    String::from("tests.test_calc.test_passes"),
                    Outcome::Passed,
                    None
                ),
                (
                    String::from("tests.test_calc.test_fails"),
                    Outcome::Failed,
                    Some(String::from("assert 3 == 4"))
                ),
                (
                    String::from("tests.test_calc.test_skipped"),
                    Outcome::NotExecuted,
                    Some(String::from("not today"))
                ),
                (
                    String::from("tests.test_calc.TestBox.test_p(1.5-x)"),
                    Outcome::Passed,
                    None
                ),
            ]
        );
    }

    #[test]
    fn a_node_id_and_its_junit_name_are_the_same_fqn() {
        let from_id: Fqn = fqn_of("tests/test_calc.py::TestBox::test_p[1.5-x]").unwrap();
        let from_junit: SpecResult = results(REPORT).unwrap().into_iter().last().unwrap();
        assert_eq!(from_id, from_junit.fqn);
    }

    #[test]
    fn an_fqn_maps_back_to_the_node_id_of_its_file() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("tests/test_calc.py"), "").unwrap();
        let fqn: Fqn = fqn_of("tests/test_calc.py::TestBox::test_p[1.5-x]").unwrap();
        assert_eq!(
            node_id_of(dir.path(), &fqn).as_deref(),
            Some("tests/test_calc.py::TestBox::test_p[1.5-x]")
        );
    }
}
