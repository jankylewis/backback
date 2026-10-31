//! `bb bench` end to end: one table plus one JSON line per item.

#[cfg(test)]
mod common;

use bench_run_helpers::{bench_with_solution, json_lines};
use common::{bb, Run};

#[cfg(test)]
mod bench_run_helpers {
    use std::path::PathBuf;

    use crate::common::{bb, harness, Run};

    pub fn bench_with_solution() -> Run {
        let sln: PathBuf = harness().join("Harness.slnx");
        let run: Run = bb(&["bench", sln.to_str().unwrap()]);
        assert_eq!(run.code, Some(0), "{}", run.all());
        run
    }

    pub fn json_lines(run: &Run) -> Vec<serde_json::Value> {
        run.out
            .lines()
            .filter(|l: &&str| l.starts_with('{'))
            .map(|l: &str| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .collect()
    }
}

const ITEMS: [&str; 7] = [
    "cold-start",
    "outline-largest",
    "outline-50k",
    "scan-harness",
    "scan-solution",
    "audit-repo",
    "binary-size",
];

#[test]
fn bench_prints_one_table_row_per_item() {
    let run: Run = bench_with_solution();
    assert!(run.out.contains("item"), "{}", run.out);
    for item in ITEMS {
        let rows: usize = run
            .out
            .lines()
            .filter(|l: &&str| l.starts_with(&format!("{item} ")))
            .count();
        assert_eq!(rows, 1, "{item} rows:\n{}", run.out);
    }
}

#[test]
fn bench_json_lines_carry_median_p95_and_five_runs() {
    let run: Run = bench_with_solution();
    let lines: Vec<serde_json::Value> = json_lines(&run);
    let names: Vec<&str> = lines
        .iter()
        .map(|v: &serde_json::Value| v["item"].as_str().unwrap())
        .collect();
    assert_eq!(names, ITEMS.to_vec(), "{}", run.out);
    for line in &lines {
        let median: u64 = line["median"].as_u64().unwrap();
        let p95: u64 = line["p95"].as_u64().unwrap();
        assert!(median > 0, "{line}");
        assert!(p95 >= median, "{line}");
        assert_eq!(line["runs"].as_u64(), Some(5), "{line}");
        assert!(line["unit"] == "us" || line["unit"] == "bytes", "{line}");
    }
}

#[test]
fn bench_without_a_solution_skips_only_the_solution_row() {
    let run: Run = bb(&["bench"]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    let names: Vec<String> = json_lines(&run)
        .iter()
        .map(|v: &serde_json::Value| v["item"].as_str().unwrap().to_owned())
        .collect();
    let expected: Vec<&str> = ITEMS
        .iter()
        .copied()
        .filter(|i: &&str| *i != "scan-solution")
        .collect();
    assert_eq!(names, expected, "{}", run.out);
}

#[test]
fn bench_rejects_a_second_argument() {
    let run: Run = bb(&["bench", "A.slnx", "extra"]);
    assert_eq!(run.code, Some(2), "{}", run.all());
    assert!(run.err.contains("usage:"), "{}", run.err);
}

#[test]
fn bench_runs_flag_sets_the_run_count_and_rows_carry_the_load() {
    let run: Run = bb(&["bench", "--runs", "7"]);
    assert_eq!(run.code, Some(0), "{}", run.all());
    let lines: Vec<serde_json::Value> = json_lines(&run);
    assert!(!lines.is_empty(), "{}", run.out);
    for line in &lines {
        assert_eq!(line["runs"].as_u64(), Some(7), "{line}");
        assert!(
            line["load_before"].as_f64().is_some_and(|l: f64| l > 0.0),
            "{line}"
        );
        assert!(
            line["load_after"].as_f64().is_some_and(|l: f64| l > 0.0),
            "{line}"
        );
    }
}

#[test]
fn bench_rejects_a_bad_run_count() {
    for bad in ["0", "x", "1000"] {
        let run: Run = bb(&["bench", "--runs", bad]);
        assert_eq!(run.code, Some(2), "--runs {bad}: {}", run.all());
    }
}
