use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
use bb_testkit::Outcome;
use bb_testkit::{Fqn, SpecList, SpecResult, Verdict};

#[derive(Clone, Debug, Default)]
pub(crate) struct TestModel {
    projects: BTreeMap<PathBuf, ProjectTests>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ProjectTests {
    pub(crate) specs: BTreeMap<Fqn, Option<SpecResult>>,
    pub(crate) expanded: bool,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
}

impl TestModel {
    pub(crate) fn add_project(&mut self, path: PathBuf) {
        self.projects.entry(path).or_default();
    }

    pub(crate) fn projects(&self) -> &BTreeMap<PathBuf, ProjectTests> {
        &self.projects
    }

    pub(crate) fn project_mut(&mut self, path: &Path) -> Option<&mut ProjectTests> {
        self.projects.get_mut(path)
    }

    pub(crate) fn merge(&mut self, path: &Path, list: SpecList) {
        let project: &mut ProjectTests = self.projects.entry(path.to_path_buf()).or_default();
        let mut previous: BTreeMap<Fqn, Option<SpecResult>> = std::mem::take(&mut project.specs);
        project.specs = list
            .specs
            .into_iter()
            .map(|fqn: Fqn| {
                let old: Option<SpecResult> = previous.remove(&fqn).flatten();
                (fqn, old)
            })
            .collect();
        project.loading = false;
        project.error = None;
    }

    pub(crate) fn record(&mut self, path: &Path, result: SpecResult) {
        if let Some(project) = self.projects.get_mut(path) {
            if let Some(slot) = project.specs.get_mut(&result.fqn) {
                *slot = Some(result);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn outcome(&self, path: &Path, name: &str) -> Option<Outcome> {
        let fqn: Fqn = Fqn::new(name).ok()?;
        self.projects
            .get(path)?
            .specs
            .get(&fqn)?
            .as_ref()
            .map(|result: &SpecResult| result.outcome)
    }

    pub(crate) fn counts(&self, path: &Path) -> (usize, usize, usize, usize) {
        let mut counts: (usize, usize, usize, usize) = (0, 0, 0, 0);
        if let Some(project) = self.projects.get(path) {
            for result in project.specs.values() {
                match result.as_ref().map(|r: &SpecResult| r.outcome.verdict()) {
                    Some(Verdict::Pass) => counts.0 += 1,
                    Some(Verdict::Fail) => counts.1 += 1,
                    Some(Verdict::Skip) => counts.2 += 1,
                    None => counts.3 += 1,
                }
            }
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rediscovery_preserves_results_only_for_surviving_specs() {
        let mut model: TestModel = TestModel::default();
        let project: PathBuf = PathBuf::from("Fixture.csproj");
        model.merge(&project, listing(&["N.C.A", "N.C.B"]));
        model.record(&project, result("N.C.A", Outcome::Failed));
        model.merge(&project, listing(&["N.C.A", "N.C.C"]));
        assert_eq!(model.outcome(&project, "N.C.A"), Some(Outcome::Failed));
        assert_eq!(model.outcome(&project, "N.C.B"), None);
        assert_eq!(model.outcome(&project, "N.C.C"), None);
        assert_eq!(model.counts(&project), (0, 1, 0, 1));
    }

    fn listing(names: &[&str]) -> SpecList {
        SpecList {
            project: String::from("Fixture"),
            assembly: PathBuf::from("Fixture.dll"),
            specs: names
                .iter()
                .map(|name: &&str| Fqn::new(name).unwrap())
                .collect(),
            elapsed: Duration::ZERO,
        }
    }

    fn result(name: &str, outcome: Outcome) -> SpecResult {
        SpecResult {
            fqn: Fqn::new(name).unwrap(),
            outcome,
            duration: Duration::from_millis(8),
            message: None,
            stack: None,
        }
    }
}
