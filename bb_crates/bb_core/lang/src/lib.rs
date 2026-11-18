//! The languages a workspace window serves (BBEOD26): C#, Rust and Python.
//!
//! The start page ticks a [`LangSet`]; [`detect`] finds which languages a
//! folder holds; [`Langs`] keeps both and answers every "may this start?"
//! question from one place: only a language that is ticked AND present is
//! active, and only an active language starts a tool or colours a file.

mod bb_plan;
mod detect;
#[cfg(feature = "fixtures")]
pub mod fixtures;
mod store;

use std::path::Path;

pub use bb_plan::{default_ticks, FileMode, Langs, Tool};
pub use detect::{detect, detect_until, Detection, DETECT_BUDGET};
pub use store::{choice_for, decode, encode, with_choice, Choices, StoreError};

/// A language backback knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {
    /// C#: Roslyn, netcoredbg and the tests panel.
    CSharp,
    /// Rust: rust-analyzer and cargo (BBEOD27); no debugger, ever.
    Rust,
    /// Python: pyright and pytest (BBEOD28); no debugger, ever.
    Python,
}

impl Lang {
    /// Every language, in the start page's order.
    pub const ALL: [Self; 3] = [Self::CSharp, Self::Rust, Self::Python];

    /// The name people read.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::CSharp => "C#",
            Self::Rust => "Rust",
            Self::Python => "Python",
        }
    }

    /// The word the saved choice uses.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::CSharp => "csharp",
            Self::Rust => "rust",
            Self::Python => "python",
        }
    }

    /// The language of a saved word; None for a word backback never wrote.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|lang: &Self| lang.key() == key)
    }

    /// The language of a source file, by its extension (case ignored).
    #[must_use]
    pub fn of_source(path: &Path) -> Option<Self> {
        let ext: String = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "cs" => Some(Self::CSharp),
            "rs" => Some(Self::Rust),
            "py" | "pyi" => Some(Self::Python),
            _ => None,
        }
    }

    /// The language a file shows the folder holds: a source file, or a
    /// project file ([`Lang::of_project_file`]).
    #[must_use]
    pub fn of_marker(path: &Path) -> Option<Self> {
        Self::of_project_file(path).or_else(|| Self::of_source(path))
    }

    /// The language of a project file: `.sln`, `.slnx`, `.csproj`,
    /// `Cargo.toml`, `pyproject.toml`, `setup.py`, `requirements*.txt` (the
    /// Explorer marks them, BBEOD28).
    #[must_use]
    pub fn of_project_file(path: &Path) -> Option<Self> {
        let name: String = path.file_name()?.to_str()?.to_ascii_lowercase();
        let ext: Option<String> = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase);
        if matches!(ext.as_deref(), Some("sln" | "slnx" | "csproj")) {
            return Some(Self::CSharp);
        }
        if name == "cargo.toml" {
            return Some(Self::Rust);
        }
        let requirements: bool = name.starts_with("requirements") && ext.as_deref() == Some("txt");
        (requirements || name == "pyproject.toml" || name == "setup.py").then_some(Self::Python)
    }
}

/// A set of languages: the ticks, the languages found, or the active ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LangSet {
    csharp: bool,
    rust: bool,
    python: bool,
}

impl LangSet {
    /// No language.
    pub const EMPTY: Self = Self {
        csharp: false,
        rust: false,
        python: false,
    };

    /// The set of `langs`.
    #[must_use]
    pub fn of(langs: &[Lang]) -> Self {
        langs
            .iter()
            .fold(Self::EMPTY, |set: Self, lang: &Lang| set.with(*lang))
    }

    /// Whether `lang` is in the set.
    #[must_use]
    pub fn has(self, lang: Lang) -> bool {
        match lang {
            Lang::CSharp => self.csharp,
            Lang::Rust => self.rust,
            Lang::Python => self.python,
        }
    }

    /// The set with `lang` in or out.
    #[must_use]
    pub fn set(mut self, lang: Lang, on: bool) -> Self {
        match lang {
            Lang::CSharp => self.csharp = on,
            Lang::Rust => self.rust = on,
            Lang::Python => self.python = on,
        }
        self
    }

    /// The set with `lang` in.
    #[must_use]
    pub fn with(self, lang: Lang) -> Self {
        self.set(lang, true)
    }

    /// The set with `lang` flipped: a tick clicked.
    #[must_use]
    pub fn toggled(self, lang: Lang) -> Self {
        self.set(lang, !self.has(lang))
    }

    /// The languages in both sets.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        self.keep(|lang: Lang| other.has(lang))
    }

    /// The languages in this set and not in `other`.
    #[must_use]
    pub fn but_not(self, other: Self) -> Self {
        self.keep(|lang: Lang| !other.has(lang))
    }

    fn keep(self, wanted: impl Fn(Lang) -> bool) -> Self {
        Self::of(
            &self
                .langs()
                .filter(|lang: &Lang| wanted(*lang))
                .collect::<Vec<Lang>>(),
        )
    }

    /// Whether no language is in the set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// The languages in the set, in [`Lang::ALL`] order.
    pub fn langs(self) -> impl Iterator<Item = Lang> {
        Lang::ALL
            .into_iter()
            .filter(move |lang: &Lang| self.has(*lang))
    }

    /// The seven sets with at least one language.
    #[must_use]
    pub fn non_empty() -> Vec<Self> {
        let mut all: Vec<Self> = vec![Self::EMPTY];
        for lang in Lang::ALL {
            let more: Vec<Self> = all.iter().map(|set: &Self| set.with(lang)).collect();
            all.extend(more);
        }
        all.retain(|set: &Self| !set.is_empty());
        all
    }

    /// The names, joined as the status bar shows them (`C# · Rust`).
    #[must_use]
    pub fn names(self) -> String {
        self.langs()
            .map(Lang::name)
            .collect::<Vec<&str>>()
            .join(" · ")
    }
}
