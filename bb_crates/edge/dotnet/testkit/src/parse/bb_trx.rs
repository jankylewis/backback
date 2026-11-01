//! TRX (Visual Studio test results XML), as the Apex Testkit reads it:
//! `TestRun/Results/UnitTestResult` joined by `testId` to
//! `TestDefinitions/UnitTest/TestMethod` (`className` + `name` = the name).

use std::collections::HashMap;
use std::time::Duration;

use roxmltree::{Document, Node};

use crate::model::{Fqn, FqnError, Outcome, SpecResult};

/// Why a TRX text cannot be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TrxError {
    /// Not well-formed XML.
    #[error("TRX is not XML: {0}")]
    Xml(String),
    /// The root is not `TestRun`, or it has no `Results`.
    #[error("TRX has no TestRun/Results")]
    NoResults,
    /// An element lacks an attribute the reader needs.
    #[error("TRX {element} has no {attribute}")]
    Missing {
        /// The element.
        element: &'static str,
        /// The attribute.
        attribute: &'static str,
    },
    /// A result's `testId` has no test definition.
    #[error("TRX result {0} has no test definition")]
    UnknownTest(String),
    /// A definition's class and name do not make a test name.
    #[error("TRX test definition: {0}")]
    Name(#[from] FqnError),
    /// An outcome backback does not know.
    #[error("TRX outcome {value:?} of {fqn} is unknown")]
    UnknownOutcome {
        /// The test.
        fqn: Fqn,
        /// The outcome text.
        value: String,
    },
    /// A duration that is not `[d.]hh:mm:ss[.fffffff]`.
    #[error("TRX duration {0:?} is not hh:mm:ss.fffffff")]
    BadDuration(String),
}

/// Every result of a TRX text, in file order.
///
/// # Errors
/// [`TrxError`] for bad XML, no `TestRun/Results`, an unknown test id or
/// outcome, or a bad duration.
pub fn parse_trx(text: &str) -> Result<Vec<SpecResult>, TrxError> {
    let text: &str = text.strip_prefix('\u{feff}').unwrap_or(text);
    let doc: Document<'_> =
        Document::parse(text).map_err(|e: roxmltree::Error| TrxError::Xml(e.to_string()))?;
    let run: Node<'_, '_> = doc.root_element();
    if !run.has_tag_name("TestRun") {
        return Err(TrxError::NoResults);
    }
    let results: Node<'_, '_> = child(run, "Results").ok_or(TrxError::NoResults)?;
    let names: HashMap<&str, Fqn> = definitions(run)?;
    results
        .children()
        .filter(|n: &Node<'_, '_>| n.has_tag_name("UnitTestResult"))
        .map(|n: Node<'_, '_>| result(n, &names))
        .collect()
}

fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children()
        .find(|n: &Node<'_, '_>| n.has_tag_name(name))
}

fn attribute<'a>(
    node: Node<'a, '_>,
    element: &'static str,
    name: &'static str,
) -> Result<&'a str, TrxError> {
    node.attribute(name).ok_or(TrxError::Missing {
        element,
        attribute: name,
    })
}

/// `UnitTest.id` to `TestMethod.className + "." + TestMethod.name`.
fn definitions<'a>(run: Node<'a, '_>) -> Result<HashMap<&'a str, Fqn>, TrxError> {
    let mut names: HashMap<&'a str, Fqn> = HashMap::new();
    let Some(defs): Option<Node<'a, '_>> = child(run, "TestDefinitions") else {
        return Ok(names);
    };
    for test in defs
        .children()
        .filter(|n: &Node<'_, '_>| n.has_tag_name("UnitTest"))
    {
        let id: &str = attribute(test, "UnitTest", "id")?;
        let method: Node<'_, '_> = child(test, "TestMethod").ok_or(TrxError::Missing {
            element: "UnitTest",
            attribute: "TestMethod",
        })?;
        let class: &str = attribute(method, "TestMethod", "className")?;
        let name: &str = attribute(method, "TestMethod", "name")?;
        names.insert(id, Fqn::new(&format!("{class}.{name}"))?);
    }
    Ok(names)
}

fn result(node: Node<'_, '_>, names: &HashMap<&str, Fqn>) -> Result<SpecResult, TrxError> {
    let id: &str = attribute(node, "UnitTestResult", "testId")?;
    let fqn: Fqn = names
        .get(id)
        .cloned()
        .ok_or_else(|| TrxError::UnknownTest(id.to_owned()))?;
    let value: &str = attribute(node, "UnitTestResult", "outcome")?;
    let Some(outcome): Option<Outcome> = Outcome::parse(value) else {
        return Err(TrxError::UnknownOutcome {
            fqn,
            value: value.to_owned(),
        });
    };
    let duration: Duration = match node.attribute("duration") {
        Some(text) => parse_duration(text).ok_or_else(|| TrxError::BadDuration(text.to_owned()))?,
        None => Duration::ZERO,
    };
    let error: Option<Node<'_, '_>> =
        child(node, "Output").and_then(|o: Node<'_, '_>| child(o, "ErrorInfo"));
    Ok(SpecResult {
        message: error_text(error, "Message"),
        stack: error_text(error, "StackTrace"),
        fqn,
        outcome,
        duration,
    })
}

/// The text of `ErrorInfo/<name>`, if any.
fn error_text(error: Option<Node<'_, '_>>, name: &str) -> Option<String> {
    error
        .and_then(|e: Node<'_, '_>| child(e, name))
        .and_then(|n: Node<'_, '_>| n.text())
        .map(str::to_owned)
}

/// A .NET `TimeSpan` in constant format: `[d.]hh:mm:ss[.fffffff]`.
#[must_use]
pub fn parse_duration(text: &str) -> Option<Duration> {
    let (days, clock): (u64, &str) = match text.split_once('.') {
        Some((d, rest)) if !d.contains(':') => (d.parse::<u64>().ok()?, rest),
        Some(_) | None => (0, text),
    };
    let (hms, fraction): (&str, &str) = clock.split_once('.').unwrap_or((clock, ""));
    let mut fields: std::str::Split<'_, char> = hms.split(':');
    let (h, m, s): (&str, &str, &str) = (fields.next()?, fields.next()?, fields.next()?);
    if fields.next().is_some() || fraction.len() > 7 || !digits(fraction) {
        return None;
    }
    let hours: u64 = bounded(h, 23)?;
    let minutes: u64 = bounded(m, 59)?;
    let seconds: u64 = bounded(s, 59)?;
    let ticks: u64 = format!("{fraction:0<7}").parse::<u64>().ok()?;
    let whole: u64 = days
        .checked_mul(24)?
        .checked_add(hours)?
        .checked_mul(60)?
        .checked_add(minutes)?
        .checked_mul(60)?
        .checked_add(seconds)?;
    let nanos: u32 = u32::try_from(ticks.checked_mul(100)?).ok()?;
    Some(Duration::new(whole, nanos))
}

fn digits(text: &str) -> bool {
    text.bytes().all(|b: u8| b.is_ascii_digit())
}

fn bounded(text: &str, max: u64) -> Option<u64> {
    if text.len() != 2 || !digits(text) {
        return None;
    }
    text.parse::<u64>().ok().filter(|v: &u64| *v <= max)
}
