mod actions;
#[path = "bb_model.rs"]
pub(crate) mod model;
#[path = "actions/navigation.rs"]
pub(crate) mod navigation;
mod view;
#[cfg(test)]
#[path = "view/window_tests.rs"]
mod window_tests;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bb_dotnet::{Capability, ProjectInfo, ProjectPath};
use bb_testkit::{ApexSettings, Fqn, Selection, SpecResult, StopFlag};

pub(crate) use actions::open;
pub(crate) use actions::toggle_project;
pub(crate) use actions::{run_category, run_class, run_failed, run_project, run_spec, stop};
pub(crate) use model::TestModel;
pub(crate) use view::panel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Files,
    Tests,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Input {
    None,
    Filter,
    Category,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scan {
    NotStarted,
    Running,
    Done,
}

pub(crate) struct TestsState {
    pub(crate) model: TestModel,
    pub(crate) side: Side,
    pub(crate) filter: String,
    pub(crate) input: Input,
    pub(crate) category: String,
    pub(crate) settings_choices: Vec<PathBuf>,
    pub(crate) settings_index: usize,
    pub(crate) busy: Option<PathBuf>,
    pub(crate) running_specs: BTreeMap<PathBuf, BTreeSet<Fqn>>,
    pub(crate) stop: Option<StopFlag>,
    pub(crate) selected: Option<(PathBuf, Fqn)>,
    pub(crate) detail: Option<SpecResult>,
    pub(crate) message: Option<String>,
    pub(crate) output: String,
    pub(crate) output_open: bool,
    pub(crate) scan: Scan,
}

impl TestsState {
    pub(crate) fn new() -> Self {
        Self {
            model: TestModel::default(),
            side: Side::Files,
            filter: String::new(),
            input: Input::None,
            category: String::new(),
            settings_choices: Vec::new(),
            settings_index: 0,
            busy: None,
            running_specs: BTreeMap::new(),
            stop: None,
            selected: None,
            detail: None,
            message: None,
            output: String::new(),
            output_open: false,
            scan: Scan::NotStarted,
        }
    }
}

impl TestsState {
    pub(crate) fn begin_run(&mut self, requests: &[(PathBuf, Selection)]) -> StopFlag {
        self.busy = requests
            .first()
            .map(|request: &(PathBuf, Selection)| request.0.clone());
        self.running_specs.clear();
        for (path, selection) in requests {
            if selection.category.is_some() {
                continue;
            }
            let names: BTreeSet<Fqn> = if selection.specs.is_empty() {
                self.model
                    .projects()
                    .get(path)
                    .into_iter()
                    .flat_map(|project: &model::ProjectTests| project.specs.keys().cloned())
                    .collect()
            } else {
                selection.specs.iter().cloned().collect()
            };
            self.running_specs.insert(path.clone(), names);
        }
        let stop: StopFlag = StopFlag::new();
        self.stop = Some(stop.clone());
        self.message = None;
        self.output.clear();
        stop
    }

    pub(crate) fn chosen_settings(&self) -> Option<PathBuf> {
        self.settings_index
            .checked_sub(1)
            .and_then(|index: usize| self.settings_choices.get(index))
            .cloned()
    }

    pub(crate) fn settings_label(&self) -> String {
        self.chosen_settings()
            .and_then(|path: PathBuf| {
                path.file_name()
                    .map(|name: &std::ffi::OsStr| name.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| String::from("Workspace default"))
    }
}

pub(crate) fn apex_settings(root: &Path) -> Result<ApexSettings, String> {
    ApexSettings::load(root).map_err(|error: bb_testkit::SettingsError| error.to_string())
}

pub(crate) fn uses_apex(path: &Path) -> bool {
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    ProjectInfo::load(&project).is_ok_and(|info: ProjectInfo| info.has(Capability::Apex))
}
