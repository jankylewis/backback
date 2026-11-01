use bb_dotnet::{
    Capability, ProjectError, ProjectInfo, ProjectPath, Solution, SolutionError, SolutionPath,
};
use bb_testkit::ApexSettings;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn scan(root: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let settings: ApexSettings = super::super::apex_settings(root)?;
    let allowed: Option<Vec<PathBuf>> = allowed_projects(settings.solution.as_deref())?;
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut projects: Vec<PathBuf> = Vec::new();
    let mut settings_paths: Vec<PathBuf> = Vec::new();
    while let Some(dir) = pending.pop() {
        let entries: fs::ReadDir = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if entry
                .file_type()
                .is_ok_and(|kind: fs::FileType| kind.is_symlink())
            {
                continue;
            }
            let path: PathBuf = entry.path();
            if path.is_dir() {
                if !skip_directory(&path) {
                    pending.push(path);
                }
            } else if path
                .extension()
                .is_some_and(|ext: &std::ffi::OsStr| ext == "runsettings")
            {
                settings_paths.push(path);
            } else if path
                .extension()
                .is_some_and(|ext: &std::ffi::OsStr| ext == "csproj")
            {
                let project: ProjectPath = ProjectPath::from_path(path.clone());
                let info: ProjectInfo =
                    ProjectInfo::load(&project).map_err(|error: ProjectError| error.to_string())?;
                if info.is_test
                    && (!info.has(Capability::Apex)
                        || allowed
                            .as_ref()
                            .is_none_or(|projects: &Vec<PathBuf>| projects.contains(&path)))
                {
                    projects.push(path);
                }
            }
        }
    }
    projects.sort();
    settings_paths.sort();
    Ok((projects, settings_paths))
}

fn skip_directory(path: &Path) -> bool {
    let name: &str = path
        .file_name()
        .and_then(|name: &std::ffi::OsStr| name.to_str())
        .unwrap_or_default();
    matches!(name, "bin" | "obj" | "node_modules" | ".git" | ".venv")
}

fn allowed_projects(path: Option<&Path>) -> Result<Option<Vec<PathBuf>>, String> {
    path.map(|path: &Path| {
        let solution_path: SolutionPath = SolutionPath::new(path.to_path_buf())
            .map_err(|error: SolutionError| error.to_string())?;
        let solution: Solution =
            Solution::load(&solution_path).map_err(|error: SolutionError| error.to_string())?;
        Ok(solution
            .projects
            .into_iter()
            .map(|project: ProjectPath| project.as_path().to_path_buf())
            .collect())
    })
    .transpose()
}
