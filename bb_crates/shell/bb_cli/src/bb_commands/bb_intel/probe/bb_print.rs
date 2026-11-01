//! Plain lines for a probe answer. Positions print 1-based (line, UTF-16
//! column + 1), paths relative to the solution folder.

use std::path::Path;
use std::time::Duration;

use bb_intel::{DocUri, FileChange, Location, Span};

use super::Answer;
use crate::bb_commands::bench::stats::{micros, Summary};

/// Most completion labels printed.
const MAX_LABELS: usize = 20;

/// `first` is the first request after the open; `warm` is every later one.
pub(super) fn timings(samples: &[Duration]) {
    let Some((first, rest)): Option<(&Duration, &[Duration])> = samples.split_first() else {
        return;
    };
    println!("first     : {first:?}");
    let warm: Vec<u64> = rest.iter().map(|d: &Duration| micros(*d)).collect();
    match Summary::of(&warm) {
        Some(s) => println!(
            "warm      : n={} p50 {:?} p95 {:?}",
            s.count,
            Duration::from_micros(s.median),
            Duration::from_micros(s.p95)
        ),
        None => println!("warm      : none (use --repeat N)"),
    }
}

pub(super) fn answer(answer: &Answer, root: &Path) {
    match answer {
        Answer::Completion(items) => {
            println!("result    : {} items", items.len());
            for item in items.iter().take(MAX_LABELS) {
                println!("  {}", item.label);
            }
        }
        Answer::Hover(None) => println!("result    : no hover"),
        Answer::Hover(Some(info)) => {
            println!("result    : hover");
            for line in info.text.lines() {
                println!("  {line}");
            }
        }
        Answer::Locations(found) => {
            println!("result    : {} locations", found.len());
            for l in found {
                println!("  {}", at(l, root));
            }
        }
        Answer::Rename(plan) => {
            println!("result    : {} files", plan.len());
            for step in plan {
                println!("  {}", change(step, root));
            }
        }
        Answer::Actions(list) => {
            println!("result    : {} actions", list.len());
            for a in list {
                println!("  {} [{}]", a.title, a.kind.as_deref().unwrap_or("-"));
            }
        }
        Answer::Diagnostics(list) => {
            println!("result    : {} diagnostics", list.len());
            for d in list {
                let code: &str = d.code.as_deref().unwrap_or("-");
                let level: String = d
                    .severity
                    .map_or_else(|| "-".to_owned(), |s: bb_intel::Severity| format!("{s:?}"));
                println!("  {} {level} {code} {}", point(d.span), d.message);
            }
        }
    }
}

fn at(l: &Location, root: &Path) -> String {
    format!("{}:{}", name(&l.uri, root), point(l.span))
}

fn change(step: &FileChange, root: &Path) -> String {
    match step {
        FileChange::Edit { uri, changes } => {
            format!("{} ({} edits)", name(uri, root), changes.len())
        }
        FileChange::Create(uri) => format!("create {}", name(uri, root)),
        FileChange::Rename { from, to } => {
            format!("rename {} -> {}", name(from, root), name(to, root))
        }
        FileChange::Delete(uri) => format!("delete {}", name(uri, root)),
    }
}

fn point(span: Span) -> String {
    format!(
        "{}:{}",
        span.start.line.saturating_add(1),
        span.start.character.saturating_add(1)
    )
}

fn name(uri: &DocUri, root: &Path) -> String {
    match uri {
        DocUri::File(p) => p
            .as_path()
            .strip_prefix(root)
            .unwrap_or_else(|_: std::path::StripPrefixError| p.as_path())
            .display()
            .to_string(),
        DocUri::Virtual(v) => v.clone(),
    }
}
