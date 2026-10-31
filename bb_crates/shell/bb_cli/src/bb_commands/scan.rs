//! `bb scan`: what every project of a solution is, for test-automation work.
//! Never prints a secret: runsettings secrets are counted, not shown.

use std::collections::BTreeMap;
use std::path::Path;

use bb_dotnet::bb_project::{EnvValue, EnvVar, RunSettings};
use bb_dotnet::testing::TestPlatform;
use bb_dotnet::{Capability, ProjectError, ProjectInfo, Solution, SolutionPath};

use super::CommandError;

pub(crate) fn run(solution: &Path) -> Result<(), CommandError> {
    let sln: SolutionPath = SolutionPath::new(solution.to_path_buf())?;
    let listing: Solution = Solution::load(&sln)?;
    println!(
        "solution  : {} ({:?}, {} projects)",
        sln.as_path().display(),
        sln.format(),
        listing.projects.len()
    );
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    let mut infos: Vec<ProjectInfo> = Vec::new();
    let mut failed: usize = 0;
    for loaded in projects(&listing) {
        match loaded {
            Ok(info) => infos.push(info),
            Err(e) => {
                eprintln!("bb: {e}");
                failed = failed.saturating_add(1);
            }
        }
    }
    let width: usize = infos
        .iter()
        .map(|i: &ProjectInfo| i.name.len())
        .max()
        .unwrap_or(0);
    for info in &infos {
        println!("{}", describe(info, width));
        count(info, &mut totals);
    }
    let summary: Vec<String> = totals
        .iter()
        .map(|(k, v): (&String, &usize)| format!("{k} {v}"))
        .collect();
    println!("summary   : {}", summary.join(" · "));
    match failed {
        0 => Ok(()),
        _ => Err(CommandError::Projects {
            failed,
            total: listing.projects.len(),
        }),
    }
}

/// Every project of `listing`, read the way `bb scan` reads it.
fn projects(listing: &Solution) -> Vec<Result<ProjectInfo, ProjectError>> {
    listing.projects.iter().map(ProjectInfo::load).collect()
}

/// The solution and every project, silently: the work `bb scan` does,
/// without printing (for `bb bench`). The first unreadable project fails it.
pub(crate) fn load(sln: &SolutionPath) -> Result<Vec<ProjectInfo>, CommandError> {
    let listing: Solution = Solution::load(sln)?;
    let infos: Vec<ProjectInfo> = projects(&listing)
        .into_iter()
        .collect::<Result<Vec<ProjectInfo>, ProjectError>>()?;
    Ok(infos)
}

/// One line per project, e.g. `MbDesktop  test exe VSTest  NUnit Apex  rs: 14 vars ...`.
pub(crate) fn describe(info: &ProjectInfo, width: usize) -> String {
    let kind: &str = if info.is_test { "test" } else { "lib " };
    let output: &str = if info.is_exe { "exe" } else { "dll" };
    let platform: String = if info.is_test {
        format!("{:?}", TestPlatform::of(info))
    } else {
        "-".to_owned()
    };
    let caps: Vec<String> = info
        .capabilities
        .iter()
        .map(|c: &Capability| format!("{c:?}"))
        .collect();
    let settings: String = info
        .runsettings
        .as_ref()
        .map_or_else(String::new, runsettings_line);
    format!(
        "  {:<width$} {kind} {output} {platform:<6} {:<28} {settings}",
        info.name,
        caps.join(" ")
    )
}

fn runsettings_line(rs: &RunSettings) -> String {
    let file: String = rs
        .path
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let secrets: usize = rs
        .env
        .iter()
        .filter(|v: &&EnvVar| v.value == EnvValue::Secret)
        .count();
    let unset: Vec<&str> = rs.unset();
    let workers: String = rs
        .nunit_workers
        .map_or_else(String::new, |w: u32| format!(", workers {w}"));
    let missing: String = unset_summary(&unset);
    format!(
        "rs {file}: {} vars ({secrets} secret{workers}{missing})",
        rs.env.len()
    )
}

/// At most three names, then a count, so one line stays readable.
pub(crate) fn unset_summary(unset: &[&str]) -> String {
    const SHOWN: usize = 3;
    let head: Vec<&str> = unset.iter().take(SHOWN).copied().collect();
    let more: usize = unset.len().saturating_sub(SHOWN);
    match (head.is_empty(), more) {
        (true, _) => String::new(),
        (false, 0) => format!(", UNSET {}", head.join(",")),
        (false, n) => format!(", UNSET {} +{n} more", head.join(",")),
    }
}

fn count(info: &ProjectInfo, totals: &mut BTreeMap<String, usize>) {
    if info.is_test {
        bump(totals, "test-projects".to_owned());
    }
    for c in &info.capabilities {
        bump(totals, format!("{c:?}"));
    }
}

fn bump(totals: &mut BTreeMap<String, usize>, key: String) {
    let slot: &mut usize = totals.entry(key).or_insert(0);
    *slot = slot.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_list_is_capped_at_three_names() {
        assert_eq!(unset_summary(&[]), "");
        assert_eq!(unset_summary(&["A", "B"]), ", UNSET A,B");
        assert_eq!(
            unset_summary(&["A", "B", "C", "D", "E"]),
            ", UNSET A,B,C +2 more"
        );
    }
}
