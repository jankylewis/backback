//! Print specs as Project > Namespace > Class > Spec, with outcomes and times.

use std::time::Duration;

use bb_testkit::{Fqn, FqnParts, Leaf, Outcome, SpecList, SpecResult, Verdict};

/// One line per new namespace, class and spec, in the given order.
fn walk<'a, T, F: FnMut(&T) -> String>(
    items: &'a [T],
    name: impl Fn(&'a T) -> &'a Fqn,
    mut line: F,
) {
    let (mut namespace, mut class): (Option<&str>, Option<&str>) = (None, None);
    for item in items {
        let parts: FqnParts<'_> = name(item).parts();
        if namespace != Some(parts.namespace) {
            let shown: &str = if parts.namespace.is_empty() {
                "(global)"
            } else {
                parts.namespace
            };
            println!("  {shown}");
            namespace = Some(parts.namespace);
            class = None;
        }
        if class != Some(parts.class) {
            println!("    {}", parts.class);
            class = Some(parts.class);
        }
        println!("      {}", line(item));
    }
}

/// The listed specs of one project, grouped as in the Testkit tree.
pub(crate) fn listed(list: &SpecList, backend: &str) {
    println!(
        "{}  [{backend}] {} specs, list {}",
        list.project,
        list.specs.len(),
        time(list.elapsed)
    );
    let mut specs: Vec<&Fqn> = list.specs.iter().collect();
    specs.sort_by_key(|f: &&Fqn| (f.parts().namespace, f.parts().class));
    walk(&specs, |f: &&Fqn| *f, |f: &&Fqn| f.parts().spec.to_owned());
}

/// The leaves of one run, each with its outcome and time.
pub(crate) fn ran(leaves: &[Leaf]) {
    let mut sorted: Vec<&Leaf> = leaves.iter().collect();
    sorted.sort_by_key(|l: &&Leaf| (l.fqn().parts().namespace, l.fqn().parts().class));
    walk(
        &sorted,
        |l: &&Leaf| l.fqn(),
        |leaf: &&Leaf| match leaf {
            Leaf::Ran(r) => ran_line(r),
            Leaf::Missing(f) => format!("ERROR {}  not in TRX", f.parts().spec),
        },
    );
}

fn ran_line(r: &SpecResult) -> String {
    let word: &str = match (r.outcome.verdict(), r.outcome) {
        (Verdict::Pass, _) => "PASS ",
        (Verdict::Fail, _) => "FAIL ",
        (Verdict::Skip, Outcome::Inconclusive) => "INCON",
        (Verdict::Skip, Outcome::Passed | Outcome::Failed | Outcome::NotExecuted) => "SKIP ",
    };
    let first: &str = r
        .message
        .as_deref()
        .and_then(|m: &str| m.lines().map(str::trim).find(|l: &&str| !l.is_empty()))
        .unwrap_or_default();
    let note: String = if first.is_empty() {
        String::new()
    } else {
        format!("  {first}")
    };
    format!("{word} {}  {}{note}", r.fqn.parts().spec, time(r.duration))
}

/// A time in ms below 10 s, else in s.
pub(crate) fn time(d: Duration) -> String {
    if d < Duration::from_secs(10) {
        format!("{} ms", d.as_millis())
    } else {
        format!("{:.2} s", d.as_secs_f64())
    }
}
