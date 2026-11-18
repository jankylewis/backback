//! What a window may start and show for its languages: the one rule every
//! tool start, file colour and Debug control asks (headmate, BBEOD26).

use std::path::Path;

use crate::{Lang, LangSet};

/// A process or service backback starts for a language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    /// The Roslyn language server; starts at open.
    Roslyn,
    /// The debugger; starts on Debug only.
    Netcoredbg,
    /// The Tests panel's project scan and `dotnet test` lists.
    TestDiscovery,
    /// rust-analyzer, the Rust language server; starts at open (BBEOD27).
    RustAnalyzer,
    /// `cargo metadata` at open (the Cargo explorer); `build`, `check` and
    /// `test` on a click.
    Cargo,
    /// pyright, the Python language server; starts at open (BBEOD28).
    Pyright,
    /// pytest: the Tests panel's Python lists and runs, on a click.
    Pytest,
}

impl Tool {
    /// Every tool.
    pub const ALL: [Self; 7] = [
        Self::Roslyn,
        Self::Netcoredbg,
        Self::TestDiscovery,
        Self::RustAnalyzer,
        Self::Cargo,
        Self::Pyright,
        Self::Pytest,
    ];

    /// The language the tool serves.
    #[must_use]
    pub fn lang(self) -> Lang {
        match self {
            Self::Roslyn | Self::Netcoredbg | Self::TestDiscovery => Lang::CSharp,
            Self::RustAnalyzer | Self::Cargo => Lang::Rust,
            Self::Pyright | Self::Pytest => Lang::Python,
        }
    }

    /// Whether it starts when the window opens (the others start on a click).
    #[must_use]
    pub fn at_open(self) -> bool {
        match self {
            Self::Roslyn | Self::RustAnalyzer | Self::Cargo | Self::Pyright => true,
            Self::Netcoredbg | Self::TestDiscovery | Self::Pytest => false,
        }
    }
}

/// How an editor tab shows a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileMode {
    /// tree-sitter colours of this language.
    Colours(Lang),
    /// Plain text: no colours.
    Plain,
}

/// The boxes a folder with no saved choice starts with: the languages it
/// holds; C# alone when it holds none (an empty or notes folder still
/// opens, as before the picker). The CLI and the start page share it.
#[must_use]
pub fn default_ticks(detected: LangSet) -> LangSet {
    if detected.is_empty() {
        LangSet::of(&[Lang::CSharp])
    } else {
        detected
    }
}

/// A window's languages: the ticks and what the folder holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Langs {
    ticked: LangSet,
    detected: LangSet,
}

impl Langs {
    /// The languages of a window that ticked `ticked` on a folder holding
    /// `detected`.
    #[must_use]
    pub fn new(ticked: LangSet, detected: LangSet) -> Self {
        Self { ticked, detected }
    }

    /// The window backback was before the picker: C# ticked and present.
    #[must_use]
    pub fn csharp_only() -> Self {
        let csharp: LangSet = LangSet::of(&[Lang::CSharp]);
        Self::new(csharp, csharp)
    }

    /// The ticks.
    #[must_use]
    pub fn ticked(self) -> LangSet {
        self.ticked
    }

    /// The languages found in the folder.
    #[must_use]
    pub fn detected(self) -> LangSet {
        self.detected
    }

    /// Ticked AND present: the only languages that start or colour anything.
    #[must_use]
    pub fn active(self) -> LangSet {
        self.ticked.and(self.detected)
    }

    /// Ticked but not in the folder: the status bar says so.
    #[must_use]
    pub fn absent(self) -> LangSet {
        self.ticked.but_not(self.detected)
    }

    /// Whether `tool` may start in this window.
    #[must_use]
    pub fn allows(self, tool: Tool) -> bool {
        self.active().has(tool.lang())
    }

    /// The tools that start when the window opens, nothing else.
    #[must_use]
    pub fn tools_at_open(self) -> Vec<Tool> {
        Tool::ALL
            .into_iter()
            .filter(|tool: &Tool| tool.at_open() && self.allows(*tool))
            .collect()
    }

    /// Whether the window shows Debug (its keys, menu and buttons): only
    /// with C# active; there is no Rust or Python debugger, ever.
    #[must_use]
    pub fn debug_visible(self) -> bool {
        self.allows(Tool::Netcoredbg)
    }

    /// Whether Debug may start from `file`: a C# file, in a window that
    /// shows Debug.
    #[must_use]
    pub fn can_debug_from(self, file: &Path) -> bool {
        self.debug_visible() && Lang::of_source(file) == Some(Lang::CSharp)
    }

    /// How a tab shows `path`: the colours of its language when that
    /// language is active, else plain text. A file of no language keeps the
    /// C# colours it always had while C# is active (C# only costs what it
    /// did before the picker).
    #[must_use]
    pub fn file_mode(self, path: &Path) -> FileMode {
        let lang: Lang = Lang::of_source(path).unwrap_or(Lang::CSharp);
        if self.active().has(lang) {
            FileMode::Colours(lang)
        } else {
            FileMode::Plain
        }
    }

    /// The status bar's language line: the active languages, then a note
    /// for each ticked one the folder does not hold.
    #[must_use]
    pub fn status(self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.active().is_empty() {
            parts.push(self.active().names());
        }
        for lang in self.absent().langs() {
            parts.push(format!("No {} project here", lang.name()));
        }
        parts.join(" · ")
    }
}
