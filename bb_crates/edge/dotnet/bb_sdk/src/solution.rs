//! Solution model: `.slnx` (XML, the .NET 10 default) and legacy `.sln`.
//!
//! This is a *listing* parser: which projects a solution contains. Project
//! evaluation (references, target frameworks) stays with Roslyn and `MSBuild`.

use std::path::{Path, PathBuf};

/// Solution errors.
#[derive(Debug, thiserror::Error)]
pub enum SolutionError {
    /// Not a `.sln` or `.slnx` file.
    #[error("not a solution file (.sln or .slnx): {0}")]
    NotASolution(PathBuf),
    /// The file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// File path.
        path: PathBuf,
        /// I/O cause.
        #[source]
        source: std::io::Error,
    },
    /// `.slnx` was not well-formed XML.
    #[error("invalid .slnx XML: {0}")]
    Xml(#[from] roxmltree::Error),
}

/// Which on-disk format a solution uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionFormat {
    /// Legacy text format.
    Sln,
    /// XML format, default since .NET 10 (`dotnet new sln`).
    Slnx,
}

/// Path to a solution file. Only constructible for `.sln`/`.slnx`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionPath {
    path: PathBuf,
    format: SolutionFormat,
}

impl SolutionPath {
    /// Validate the extension. Does not touch the disk.
    ///
    /// # Errors
    /// [`SolutionError::NotASolution`] for any other extension.
    pub fn new(path: PathBuf) -> Result<Self, SolutionError> {
        let ext: String = path
            .extension()
            .and_then(|e: &std::ffi::OsStr| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let format: SolutionFormat = match ext.as_str() {
            "sln" => SolutionFormat::Sln,
            "slnx" => SolutionFormat::Slnx,
            _ => return Err(SolutionError::NotASolution(path)),
        };
        Ok(Self { path, format })
    }

    /// The path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// The format.
    #[must_use]
    pub fn format(&self) -> SolutionFormat {
        self.format
    }
}

/// Path to a project file (`.csproj` and friends).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectPath(PathBuf);

impl ProjectPath {
    /// Wrap a project path, for example one passed on the command line.
    #[must_use]
    pub fn from_path(path: PathBuf) -> Self {
        Self(path)
    }

    /// The path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// A loaded solution listing.
#[derive(Debug, Clone)]
pub struct Solution {
    /// Where it came from.
    pub path: SolutionPath,
    /// Projects in file order.
    pub projects: Vec<ProjectPath>,
}

impl Solution {
    /// Read and parse a solution file.
    ///
    /// # Errors
    /// The file cannot be read, or `.slnx` is not XML.
    pub fn load(path: &SolutionPath) -> Result<Self, SolutionError> {
        let text: String =
            std::fs::read_to_string(path.as_path()).map_err(|source: std::io::Error| {
                SolutionError::Read {
                    path: path.as_path().to_path_buf(),
                    source,
                }
            })?;
        let dir: &Path = path.as_path().parent().unwrap_or_else(|| Path::new("."));
        let projects: Vec<ProjectPath> = match path.format() {
            SolutionFormat::Slnx => parse_slnx(&text, dir)?,
            SolutionFormat::Sln => parse_sln(&text, dir),
        };
        Ok(Self {
            path: path.clone(),
            projects,
        })
    }
}

fn parse_slnx(text: &str, dir: &Path) -> Result<Vec<ProjectPath>, SolutionError> {
    let doc: roxmltree::Document<'_> = roxmltree::Document::parse(text)?;
    Ok(doc
        .descendants()
        .filter(|n: &roxmltree::Node<'_, '_>| n.has_tag_name("Project"))
        .filter_map(|n: roxmltree::Node<'_, '_>| n.attribute("Path"))
        .filter(|p: &&str| is_project_file(p))
        .map(|p: &str| resolve(dir, p))
        .collect())
}

fn parse_sln(text: &str, dir: &Path) -> Vec<ProjectPath> {
    // Project("{TYPE-GUID}") = "Name", "Rel\Path.csproj", "{PROJECT-GUID}"
    // Split on '"': ["", TYPE-GUID, ") = ", Name, ", ", Path, ...] -> the path is item 5.
    text.lines()
        .filter_map(|line: &str| line.trim_start().strip_prefix("Project("))
        .filter_map(|rest: &str| rest.split('"').nth(5))
        .filter(|p: &&str| is_project_file(p))
        .map(|p: &str| resolve(dir, p))
        .collect()
}

fn is_project_file(raw: &str) -> bool {
    // Solution folders have no extension; every MSBuild project ends in "proj".
    raw.rsplit('.')
        .next()
        .is_some_and(|ext: &str| ext.to_ascii_lowercase().ends_with("proj"))
}

fn resolve(dir: &Path, raw: &str) -> ProjectPath {
    ProjectPath(dir.join(raw.replace('\\', "/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_solution_extensions() {
        assert!(SolutionPath::new(PathBuf::from("a.csproj")).is_err());
        let slnx: SolutionPath = SolutionPath::new(PathBuf::from("A.SLNX")).unwrap();
        assert_eq!(slnx.format(), SolutionFormat::Slnx);
    }

    #[test]
    fn slnx_lists_projects_and_skips_folders() {
        let xml: &str = r#"<Solution>
  <Folder Name="/src/">
    <Project Path="src\Core\Core.csproj" />
    <Project Path="src/App/App.fsproj" />
  </Folder>
  <Folder Name="/docs/" />
</Solution>"#;
        let projects: Vec<ProjectPath> = parse_slnx(xml, Path::new("/repo")).unwrap();
        let shown: Vec<PathBuf> = projects
            .iter()
            .map(|p: &ProjectPath| p.as_path().to_path_buf())
            .collect();
        assert_eq!(
            shown,
            vec![
                PathBuf::from("/repo/src/Core/Core.csproj"),
                PathBuf::from("/repo/src/App/App.fsproj")
            ]
        );
    }

    #[test]
    fn sln_lists_projects_and_skips_solution_folders() {
        let sln: &str = r#"
Microsoft Visual Studio Solution File, Format Version 12.00
Project("{2150E333-8FDC-42A3-9474-1A3956D46DE8}") = "src", "src", "{AAAA}"
EndProject
Project("{9A19103F-16F7-4668-BE54-9A1E7A4F7556}") = "Core", "src\Core\Core.csproj", "{BBBB}"
EndProject
"#;
        let projects: Vec<ProjectPath> = parse_sln(sln, Path::new("/repo"));
        let first: Option<PathBuf> = projects
            .first()
            .map(|p: &ProjectPath| p.as_path().to_path_buf());
        assert_eq!(projects.len(), 1);
        assert_eq!(first, Some(PathBuf::from("/repo/src/Core/Core.csproj")));
    }
}
