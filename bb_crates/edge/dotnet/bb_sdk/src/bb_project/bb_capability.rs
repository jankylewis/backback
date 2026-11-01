//! Map package ids, project references and MSBuild properties to [`Capability`].

use super::{is_true, property, Capability, Facts};

pub(crate) fn capabilities_of(facts: &Facts) -> Vec<Capability> {
    let mut found: Vec<Capability> = facts
        .packages
        .iter()
        .filter_map(|p: &String| package_capability(p))
        .collect();
    let apex_ref: bool = facts.project_refs.iter().any(|r: &String| {
        let stem: String = r
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .trim_end_matches(".csproj")
            .to_ascii_lowercase();
        is_family(&stem, "apex")
    });
    if apex_ref {
        found.push(Capability::Apex);
    }
    let mtp: bool = [
        "UseMicrosoftTestingPlatformRunner",
        "TestingPlatformDotnetTestSupport",
        "EnableMSTestRunner",
        "EnableNUnitRunner",
    ]
    .iter()
    .any(|name: &&str| property(facts, name).is_some_and(is_true));
    if mtp {
        found.push(Capability::TestingPlatform);
    }
    found.sort();
    found.dedup();
    found
}

/// `id` is `family` itself or `family.<something>`.
fn is_family(id: &str, family: &str) -> bool {
    id == family
        || id
            .strip_prefix(family)
            .is_some_and(|rest: &str| rest.starts_with('.'))
}

fn package_capability(package: &str) -> Option<Capability> {
    let p: String = package.to_ascii_lowercase();
    if is_family(&p, "nunit") || p == "nunit3testadapter" {
        Some(Capability::NUnit)
    } else if is_family(&p, "xunit") {
        Some(Capability::XUnit)
    } else if is_family(&p, "mstest") {
        Some(Capability::MsTest)
    } else if is_family(&p, "microsoft.playwright") {
        Some(Capability::Playwright)
    } else if is_family(&p, "apex") {
        Some(Capability::Apex)
    } else if is_family(&p, "modelcontextprotocol") {
        Some(Capability::Mcp)
    } else if is_family(&p, "microsoft.testing.platform") {
        Some(Capability::TestingPlatform)
    } else {
        None
    }
}
