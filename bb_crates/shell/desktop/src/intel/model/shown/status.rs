//! Where the solution load stands, as the status bar shows it.

use std::path::PathBuf;

/// Where the solution load stands, as the status bar shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoadStatus {
    /// The workspace holds no solution; intel stays off.
    NoSolution,
    /// Projects loaded so far.
    Loading { loaded: usize, total: usize },
    /// Every project loaded.
    Ready,
    /// The active server connection closed.
    Stopped,
    /// The server failed; the text says why.
    Failed(String),
}

impl LoadStatus {
    /// A batch line counted `loaded` of `total` projects. Only a load in
    /// progress moves: Roslyn logs batch lines after the load too (a
    /// decompiled metadata document loads one), and they must not turn
    /// `Ready` or `Failed` back into `Loading`.
    pub(crate) fn progress(&mut self, loaded: usize, total: usize) {
        match self {
            Self::Loading { .. } => *self = Self::Loading { loaded, total },
            Self::NoSolution | Self::Ready | Self::Stopped | Self::Failed(_) => {}
        }
    }

    /// The status bar text.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::NoSolution => String::from("No solution"),
            Self::Loading { loaded, total } => format!("Loading {loaded}/{total} projects"),
            Self::Ready => String::from("Ready"),
            Self::Stopped => String::from("C# server stopped — Restart"),
            Self::Failed(reason) => format!("C# server failed: {reason}"),
        }
    }
}

impl crate::intel::model::IntelModel {
    /// The load is done; `failed` are the projects Roslyn could not load.
    pub(crate) fn ready(&mut self, failed: Vec<PathBuf>) {
        self.status = LoadStatus::Ready;
        for project in &failed {
            eprintln!("intel_failed_project={}", project.display());
        }
        self.failed_projects = failed;
    }

    /// The status bar text, failed projects included.
    pub(crate) fn status_label(&self) -> String {
        status_label(&self.status, &self.failed_projects)
    }
}

/// The status bar text: the load status, and once ready the projects
/// Roslyn could not load (their reasons are in the server log).
pub(crate) fn status_label(status: &LoadStatus, failed: &[PathBuf]) -> String {
    match status {
        LoadStatus::Ready if !failed.is_empty() => {
            let names: Vec<String> = failed
                .iter()
                .map(|p: &PathBuf| {
                    p.file_name().map_or_else(
                        || p.display().to_string(),
                        |n: &std::ffi::OsStr| n.to_string_lossy().into_owned(),
                    )
                })
                .collect();
            let noun: &str = if failed.len() == 1 {
                "project"
            } else {
                "projects"
            };
            format!(
                "Ready — {} {noun} failed to load: {}",
                failed.len(),
                names.join(", ")
            )
        }
        LoadStatus::NoSolution
        | LoadStatus::Loading { .. }
        | LoadStatus::Ready
        | LoadStatus::Stopped
        | LoadStatus::Failed(_) => status.label(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_with_failed_projects_names_them_in_the_status_bar() {
        let failed: Vec<PathBuf> = vec![
            PathBuf::from("/s/_NUnitApex/_NUnitApex.csproj"),
            PathBuf::from("/s/Core/_Core.csproj"),
        ];
        assert_eq!(status_label(&LoadStatus::Ready, &[]), "Ready");
        assert_eq!(
            status_label(&LoadStatus::Ready, &failed),
            "Ready — 2 projects failed to load: _NUnitApex.csproj, _Core.csproj"
        );
        assert_eq!(
            status_label(&LoadStatus::Stopped, &failed),
            "C# server stopped — Restart"
        );
    }
}
