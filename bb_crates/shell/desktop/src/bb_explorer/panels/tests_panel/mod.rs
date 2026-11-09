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
pub(crate) use view::{open_spec, panel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Files,
    Tests,
}

/// A row under a project that folds its children away (BB-BUG-024).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Group {
    Namespace(String),
    /// `namespace.class`.
    Class(String),
    /// A parameterised method's name without arguments.
    Method(String),
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
    /// A test's file and line to go to once its tab has loaded.
    pub(crate) pending_jump: Option<(PathBuf, usize)>,
    pub(crate) output_open: bool,
    pub(crate) scan: Scan,
    /// Folded groups, per project.
    pub(crate) collapsed: BTreeSet<(PathBuf, Group)>,
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
            pending_jump: None,
            output_open: false,
            scan: Scan::NotStarted,
            collapsed: BTreeSet::new(),
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

    /// Fold `group` if open, open it if folded.
    pub(crate) fn toggle(&mut self, path: &Path, group: Group) {
        let key: (PathBuf, Group) = (path.to_path_buf(), group);
        if !self.collapsed.remove(&key) {
            self.collapsed.insert(key);
        }
    }

    pub(crate) fn is_collapsed(&self, path: &Path, group: &Group) -> bool {
        self.collapsed
            .contains(&(path.to_path_buf(), group.clone()))
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

/// The Apex CLI runs `path` only when it references Apex and the workspace
/// `root` opts in with `apexTestkit.*` keys, as `bb test` decides. Released
/// Apex CLIs ignore `--list` and `--specs`, so an Apex project without the
/// keys lists and runs through `dotnet test` (BB-BUG-061). A settings file
/// that cannot be read still picks Apex, so its error is shown.
pub(crate) fn uses_apex(root: &Path, path: &Path) -> bool {
    let project: ProjectPath = ProjectPath::from_path(path.to_path_buf());
    let apex: bool =
        ProjectInfo::load(&project).is_ok_and(|info: ProjectInfo| info.has(Capability::Apex));
    apex && match ApexSettings::load(root) {
        Ok(settings) => settings.declared,
        Err(_) => true,
    }
}

#[cfg(test)]
mod apex_tests {
    use std::path::{Path, PathBuf};

    const APEX_PROJECT: &str = "<Project Sdk=\"Microsoft.NET.Sdk\"><ItemGroup>\
        <PackageReference Include=\"Apex\" Version=\"1.0.0\" />\
        <PackageReference Include=\"NUnit\" Version=\"4.0.0\" />\
        </ItemGroup></Project>";

    fn workspace(name: &str) -> PathBuf {
        let root: PathBuf =
            std::env::temp_dir().join(format!("bb-apexpick-{name}-{}", std::process::id()));
        let _old: std::io::Result<()> = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("Specs")).unwrap();
        std::fs::write(root.join("Specs/Specs.csproj"), APEX_PROJECT).unwrap();
        root
    }

    #[test]
    fn an_apex_project_runs_through_dotnet_test_until_the_workspace_opts_in() {
        let root: PathBuf = workspace("optin");
        let project: PathBuf = root.join("Specs/Specs.csproj");
        assert!(!super::uses_apex(&root, &project));
        std::fs::create_dir_all(root.join(".vscode")).unwrap();
        std::fs::write(
            root.join(".vscode/settings.json"),
            "{ \"apexTestkit.command\": \"dotnet apex\" }",
        )
        .unwrap();
        assert!(super::uses_apex(&root, &project));
        std::fs::write(
            root.join(".vscode/settings.json"),
            "{ \"editor.tabSize\": 4 }",
        )
        .unwrap();
        assert!(!super::uses_apex(&root, &project));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_project_without_apex_never_uses_the_apex_cli() {
        let root: PathBuf = workspace("plain");
        let plain: PathBuf = root.join("Specs/Plain.csproj");
        std::fs::write(&plain, "<Project Sdk=\"Microsoft.NET.Sdk\" />").unwrap();
        std::fs::create_dir_all(root.join(".vscode")).unwrap();
        std::fs::write(
            root.join(".vscode/settings.json"),
            "{ \"apexTestkit.command\": \"dotnet apex\" }",
        )
        .unwrap();
        assert!(!super::uses_apex(Path::new(&root), &plain));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
