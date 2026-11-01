//! Which test platform a project runs on.
//!
//! .NET has two test platforms: VSTest (legacy) and Microsoft.Testing.Platform
//! (MTP, opt-in per project). Listing and running live in `bb-testkit`:
//! `dotnet test --list-tests` prints display names, not full names, so it
//! cannot name a spec there.

use crate::bb_project::{Capability, ProjectInfo};

/// Which test platform a project uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPlatform {
    /// Legacy VSTest.
    VsTest,
    /// Microsoft.Testing.Platform: the test project is its own app.
    Mtp,
}

impl TestPlatform {
    /// The platform a detected project runs on.
    #[must_use]
    pub fn of(project: &ProjectInfo) -> Self {
        if project.has(Capability::TestingPlatform) {
            Self::Mtp
        } else {
            Self::VsTest
        }
    }
}
