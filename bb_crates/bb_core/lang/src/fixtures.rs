//! The seven fixture folders of BBEOD26 (feature `fixtures`, for tests):
//! each holds its languages' project and source files, plus decoy sources
//! of every language in a dot folder, a git-ignored folder and build
//! folders, which detection must skip.

use std::path::{Path, PathBuf};

use crate::Lang;

/// One fixture: its folder name and the languages it holds.
pub struct Fixture {
    /// The folder's name.
    pub name: &'static str,
    /// The languages it holds.
    pub langs: &'static [Lang],
}

/// `cs_only`, `rust_only`, `py_only`, `cs_rust`, `cs_py`, `py_rust`, `all3`.
pub const FIXTURES: [Fixture; 7] = [
    Fixture {
        name: "cs_only",
        langs: &[Lang::CSharp],
    },
    Fixture {
        name: "rust_only",
        langs: &[Lang::Rust],
    },
    Fixture {
        name: "py_only",
        langs: &[Lang::Python],
    },
    Fixture {
        name: "cs_rust",
        langs: &[Lang::CSharp, Lang::Rust],
    },
    Fixture {
        name: "cs_py",
        langs: &[Lang::CSharp, Lang::Python],
    },
    Fixture {
        name: "py_rust",
        langs: &[Lang::Python, Lang::Rust],
    },
    Fixture {
        name: "all3",
        langs: &[Lang::CSharp, Lang::Rust, Lang::Python],
    },
];

const DECOYS: [&str; 10] = [
    ".gitignore",
    ".hidden/Decoy.cs",
    ".hidden/decoy.rs",
    ".hidden/decoy.py",
    "ignored/Decoy.cs",
    "ignored/Cargo.toml",
    "ignored/requirements.txt",
    "target/decoy.py",
    "obj/decoy.rs",
    "notes/README.md",
];

fn files_of(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::CSharp => &["App.sln", "App/App.csproj", "App/Program.cs"],
        Lang::Rust => &["Cargo.toml", "src/main.rs"],
        Lang::Python => &["pyproject.toml", "tool/main.py"],
    }
}

fn text_of(file: &str) -> &'static str {
    match Lang::of_source(Path::new(file)) {
        Some(Lang::CSharp) => "class Program { static void Main() { } }\n",
        Some(Lang::Rust) => "fn main() { let s = \"hi\"; }\n",
        Some(Lang::Python) => "def main():\n    return \"hi\"\n",
        None if file == ".gitignore" => "ignored/\n",
        None => "x\n",
    }
}

fn source_of(lang: Lang, held: bool) -> &'static str {
    match (lang, held) {
        (Lang::CSharp, true) => "App/Program.cs",
        (Lang::Rust, true) => "src/main.rs",
        (Lang::Python, true) => "tool/main.py",
        (Lang::CSharp, false) => ".hidden/Decoy.cs",
        (Lang::Rust, false) => ".hidden/decoy.rs",
        (Lang::Python, false) => ".hidden/decoy.py",
    }
}

impl Fixture {
    /// Write the folder under `base`; its canonical path.
    ///
    /// # Errors
    /// The file system's error.
    pub fn write(&self, base: &Path) -> std::io::Result<PathBuf> {
        let root: PathBuf = base.join(self.name);
        let own: Vec<&str> = self
            .langs
            .iter()
            .flat_map(|lang: &Lang| files_of(*lang).iter().copied())
            .collect();
        for file in own.iter().chain(DECOYS.iter()) {
            let path: PathBuf = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap_or(&root))?;
            std::fs::write(&path, text_of(file))?;
        }
        root.canonicalize()
    }

    /// A source file of every language, held or decoy, with its language.
    #[must_use]
    pub fn sources(&self) -> Vec<(&'static str, Lang)> {
        Lang::ALL
            .into_iter()
            .map(|lang: Lang| (source_of(lang, self.langs.contains(&lang)), lang))
            .collect()
    }
}
