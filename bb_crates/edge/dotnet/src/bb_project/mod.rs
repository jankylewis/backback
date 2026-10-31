//! What a C# project *is*, read from its `.csproj` and the `Directory.Build.props`
//! files above it: test framework, browser stack (Apex, Playwright), MCP server,
//! test platform, output kind, runsettings.

mod capability;
mod runsettings;

use std::path::{Path, PathBuf};

pub use runsettings::{classify, EnvValue, EnvVar, RunSettings};

use crate::solution::ProjectPath;

/// A capability backback recognises in a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// NUnit (`NUnit`, `NUnit3TestAdapter`).
    NUnit,
    /// xUnit v2 or v3 (`xunit`, `xunit.v3`).
    XUnit,
    /// `MSTest` (`MSTest`, `MSTest.TestFramework`).
    MsTest,
    /// Microsoft Playwright for .NET (`Microsoft.Playwright*`).
    Playwright,
    /// Apex, MODEC's C# browser framework (package `Apex*` or a project reference).
    Apex,
    /// An MCP server or client (`ModelContextProtocol*`).
    Mcp,
    /// Runs on Microsoft.Testing.Platform instead of `VSTest`.
    TestingPlatform,
}

/// Errors reading a project.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// The project file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// File.
        path: PathBuf,
        /// Cause.
        #[source]
        source: std::io::Error,
    },
    /// The project file is not well-formed XML.
    #[error("invalid XML in {path}: {source}")]
    Xml {
        /// File.
        path: PathBuf,
        /// Cause.
        #[source]
        source: roxmltree::Error,
    },
}

/// Everything backback knows about one project.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// The project file.
    pub path: ProjectPath,
    /// File name without extension.
    pub name: String,
    /// `IsTestProject` is true, or a test framework is referenced.
    pub is_test: bool,
    /// `OutputType` is `Exe`.
    pub is_exe: bool,
    /// Recognised capabilities, sorted.
    pub capabilities: Vec<Capability>,
    /// The runsettings file the project uses, if any.
    pub runsettings: Option<RunSettings>,
}

/// Raw facts collected from the XML of one or more MSBuild files.
#[derive(Debug, Default)]
pub(crate) struct Facts {
    pub(crate) packages: Vec<String>,
    pub(crate) project_refs: Vec<String>,
    pub(crate) properties: Vec<(String, String)>,
}

impl ProjectInfo {
    /// Read `project` and the `Directory.Build.props` files above it.
    ///
    /// # Errors
    /// The project file cannot be read or is not XML.
    pub fn load(project: &ProjectPath) -> Result<Self, ProjectError> {
        let mut facts: Facts = Facts::default();
        for file in msbuild_chain(project.as_path()) {
            read_facts(&file, &mut facts)?;
        }
        let capabilities: Vec<Capability> = capability::capabilities_of(&facts);
        let test_framework: bool = capabilities.iter().any(|c: &Capability| {
            matches!(
                c,
                Capability::NUnit | Capability::XUnit | Capability::MsTest
            )
        });
        let is_test: bool =
            property(&facts, "IsTestProject").is_some_and(is_true) || test_framework;
        let is_exe: bool =
            property(&facts, "OutputType").is_some_and(|v: &str| v.eq_ignore_ascii_case("exe"));
        let dir: &Path = project.as_path().parent().unwrap_or_else(|| Path::new("."));
        let runsettings: Option<RunSettings> =
            RunSettings::find(dir, property(&facts, "RunSettingsFilePath"));
        let name: String = project
            .as_path()
            .file_stem()
            .map(|s: &std::ffi::OsStr| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Ok(Self {
            path: project.clone(),
            name,
            is_test,
            is_exe,
            capabilities,
            runsettings,
        })
    }

    /// Whether the project has a capability.
    #[must_use]
    pub fn has(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// The project file last, preceded by every `Directory.Build.props` above it
/// (outermost first), so inner files override outer ones.
fn msbuild_chain(project: &Path) -> Vec<PathBuf> {
    let mut chain: Vec<PathBuf> = project
        .ancestors()
        .skip(1)
        .map(|d: &Path| d.join("Directory.Build.props"))
        .filter(|p: &PathBuf| p.is_file())
        .collect();
    chain.reverse();
    chain.push(project.to_path_buf());
    chain
}

fn read_facts(file: &Path, facts: &mut Facts) -> Result<(), ProjectError> {
    let text: String =
        std::fs::read_to_string(file).map_err(|source: std::io::Error| ProjectError::Read {
            path: file.to_path_buf(),
            source,
        })?;
    let doc: roxmltree::Document<'_> =
        roxmltree::Document::parse(&text).map_err(|source: roxmltree::Error| {
            ProjectError::Xml {
                path: file.to_path_buf(),
                source,
            }
        })?;
    for node in doc.descendants().filter(roxmltree::Node::is_element) {
        let include: Option<&str> = node.attribute("Include");
        match (node.tag_name().name(), include) {
            ("PackageReference", Some(id)) => facts.packages.push(id.to_owned()),
            ("ProjectReference", Some(path)) => facts.project_refs.push(path.replace('\\', "/")),
            (name, _) if is_property(&node) => {
                let value: String = node.text().unwrap_or_default().trim().to_owned();
                facts.properties.push((name.to_owned(), value));
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_property(node: &roxmltree::Node<'_, '_>) -> bool {
    node.parent_element()
        .is_some_and(|p: roxmltree::Node<'_, '_>| p.has_tag_name("PropertyGroup"))
}

/// Last definition wins, as in MSBuild evaluation order.
pub(crate) fn property<'a>(facts: &'a Facts, name: &str) -> Option<&'a str> {
    facts
        .properties
        .iter()
        .rev()
        .find(|(k, _): &&(String, String)| k == name)
        .map(|(_, v): &(String, String)| v.as_str())
}

pub(crate) fn is_true(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}

#[cfg(test)]
mod tests;
