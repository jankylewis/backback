//! Properties of the readers over generated input.

use std::time::Duration;

use bb_testkit::{parse_duration, parse_trx, strip_jsonc, Fqn, FqnParts, Outcome, SpecResult};
use proptest::prelude::*;
use serde_json::Value;

const OUTCOMES: [&str; 4] = ["Passed", "Failed", "NotExecuted", "Inconclusive"];

#[cfg(test)]
mod gen {
    use super::*;
    use std::fmt::Write;

    /// A `TimeSpan` in constant format, as TRX writes it.
    pub fn timespan(whole: u64, ticks: u32) -> String {
        let (days, rest): (u64, u64) = (whole / 86_400, whole % 86_400);
        let clock: String = format!(
            "{:02}:{:02}:{:02}.{ticks:07}",
            rest / 3600,
            rest % 3600 / 60,
            rest % 60
        );
        if days == 0 {
            clock
        } else {
            format!("{days}.{clock}")
        }
    }

    pub fn ident() -> impl Strategy<Value = String> {
        "[A-Z][A-Za-z0-9_]{0,8}"
    }

    pub fn args() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(String::new()),
            "\\(\"[a-z.,() ]{0,6}\"(, [0-9.]{1,4})?\\)"
        ]
    }

    pub fn json() -> impl Strategy<Value = Value> {
        let leaf: BoxedStrategy<Value> = prop_oneof![
            any::<bool>().prop_map(Value::from),
            any::<i32>().prop_map(Value::from),
            "[a-z/*, \"\\\\]{0,8}".prop_map(Value::from),
        ]
        .boxed();
        leaf.prop_recursive(3, 24, 4, |inner: BoxedStrategy<Value>| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
                prop::collection::btree_map("[a-z/]{1,4}", inner, 0..4).prop_map(
                    |m: std::collections::BTreeMap<String, Value>| {
                        Value::Object(m.into_iter().collect())
                    }
                ),
            ]
        })
    }

    pub fn trx(rows: &[(String, String, usize, u64)]) -> String {
        let mut results: String = String::new();
        let mut defs: String = String::new();
        for (i, (class, name, outcome, ticks)) in rows.iter().enumerate() {
            let duration: String = timespan(
                ticks / 10_000_000,
                u32::try_from(ticks % 10_000_000).unwrap(),
            );
            write!(
            results,
            "<UnitTestResult testId=\"id{i}\" testName=\"x\" outcome=\"{}\" duration=\"{duration}\" />",
            OUTCOMES[*outcome]
        )
        .unwrap();
            let escaped: String = name
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;");
            write!(
            defs,
            "<UnitTest id=\"id{i}\"><TestMethod className=\"{class}\" name=\"{escaped}\" /></UnitTest>"
        )
        .unwrap();
        }
        format!("\u{feff}<TestRun xmlns=\"http://microsoft.com/schemas/VisualStudio/TeamTest/2010\"><Results>{results}</Results><TestDefinitions>{defs}</TestDefinitions></TestRun>")
    }
}

use gen::{args, ident, json, timespan, trx};

proptest! {
    #![proptest_config(ProptestConfig { failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn a_trx_duration_reads_back_to_the_same_time(whole in 0u64..2_000_000, ticks in 0u32..10_000_000) {
        let text: String = timespan(whole, ticks);
        prop_assert_eq!(parse_duration(&text), Some(Duration::new(whole, ticks * 100)));
    }

    #[test]
    fn a_name_splits_back_into_what_built_it(
        ns in prop::collection::vec(ident(), 0..4),
        class in ident(),
        method in ident(),
        tail in args(),
    ) {
        let space: String = ns.join(".");
        let raw: String = if space.is_empty() { format!("{class}.{method}{tail}") } else { format!("{space}.{class}.{method}{tail}") };
        let fqn: Fqn = Fqn::new(&raw).unwrap();
        let parts: FqnParts<'_> = fqn.parts();
        prop_assert_eq!(parts.namespace, space.as_str());
        prop_assert_eq!(parts.class, class.as_str());
        prop_assert_eq!(parts.spec, format!("{method}{tail}"));
    }

    #[test]
    fn comments_and_trailing_commas_never_change_the_value(value in json(), note in "[a-z /*\"]{0,8}") {
        let pretty: String = serde_json::to_string_pretty(&value).unwrap();
        let clean: String = note.replace('*', "-");
        let noisy: String = pretty
            .lines()
            .fold(String::new(), |mut all: String, l: &str| {
                std::fmt::Write::write_fmt(&mut all, format_args!("/* {clean} */{l} // {note}\n")).unwrap();
                all
            })
            .replace("\n}", ",\n}")
            .replace("\n]", ",\n]")
            .replace("[,", "[")
            .replace("{,", "{");
        let read: Value = serde_json::from_str(&strip_jsonc(&noisy)).unwrap();
        prop_assert_eq!(read, value);
    }

    #[test]
    fn every_trx_row_reads_back_with_its_name_outcome_and_time(
        rows in prop::collection::vec((ident(), ident(), args(), 0usize..4, 0u64..1_000_000_000_000), 0..8),
    ) {
        let rows: Vec<(String, String, usize, u64)> = rows
            .into_iter()
            .map(|(class, method, tail, outcome, ticks): (String, String, String, usize, u64)| {
                (format!("Ns.{class}"), format!("{method}{tail}"), outcome, ticks)
            })
            .collect();
        let read: Vec<SpecResult> = parse_trx(&trx(&rows)).unwrap();
        prop_assert_eq!(read.len(), rows.len());
        for (got, (class, name, outcome, ticks)) in read.iter().zip(&rows) {
            prop_assert_eq!(got.fqn.as_str(), format!("{class}.{name}"));
            prop_assert_eq!(Some(got.outcome), Outcome::parse(OUTCOMES[*outcome]));
            prop_assert_eq!(got.duration, Duration::from_nanos(ticks * 100));
        }
    }
}
