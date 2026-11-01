//! .NET side of backback: find the SDK, read solutions and projects, run
//! `dotnet` with deadlines.
//!
//! Roslyn (through `bb-lsp`) understands the code; this crate understands the
//! *project system*: `.sln`/`.slnx`/`.csproj`, runsettings, and the `dotnet` CLI.

pub mod bb_project;
pub mod sdk;
pub mod solution;
pub mod testing;

pub use bb_project::{Capability, ProjectError, ProjectInfo};
pub use sdk::{Sdk, SdkError};
pub use solution::{ProjectPath, Solution, SolutionError, SolutionFormat, SolutionPath};
