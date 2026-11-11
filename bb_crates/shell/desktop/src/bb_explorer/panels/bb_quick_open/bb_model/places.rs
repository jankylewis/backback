//! Find in Files' fields, as Wu, VS Code and Rider: the text, "in" (the
//! workspace, a C# project or a folder, picked from a list its typed name
//! narrows) and "files" (globs; `!` excludes). Tab moves between them.
//! Rider's scope buttons (Solution, Project, Directory) keep one kind of
//! place in the list, and its Cc, W and .* toggles set the search options.

use std::path::{Path, PathBuf};

use super::rank::{name_of, project_label, shown};
use super::{Item, QuickOpen};

/// The field keys go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Field {
    /// What to find.
    #[default]
    Text,
    /// Where: narrows the list of places.
    In,
    /// Which files: globs.
    Files,
}

impl Field {
    /// The field after (Tab) or before (Shift+Tab) this one.
    fn next(self, back: bool) -> Field {
        match (self, back) {
            (Field::Text, false) | (Field::Files, true) => Field::In,
            (Field::In, false) | (Field::Text, true) => Field::Files,
            (Field::Files, false) | (Field::In, true) => Field::Text,
        }
    }
}

/// What Enter in Find in Files asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindAsk {
    pub(crate) text: String,
    /// The folder searched.
    pub(crate) dir: PathBuf,
    /// Its label, for the results title.
    pub(crate) place: String,
    /// The files field, as typed.
    pub(crate) files: String,
    /// The Cc, W and .* toggles.
    pub(crate) options: bb_search::Options,
}

/// Rider's scope buttons: which kind of place the list keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Scope {
    /// Every place.
    #[default]
    All,
    /// The workspace (Rider's Solution).
    Solution,
    /// The C# projects: the list is Rider's project dropdown.
    Project,
    /// Folders.
    Directory,
}

impl Scope {
    /// The label start of this scope's places; `None` keeps every place.
    pub(crate) fn prefix(self) -> Option<&'static str> {
        match self {
            Scope::All => None,
            Scope::Solution => Some("Workspace · "),
            Scope::Project => Some("Project · "),
            Scope::Directory => Some("Folder · "),
        }
    }
}

/// One of Rider's search toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Toggle {
    /// Cc (Alt+C).
    Case,
    /// W (Alt+W).
    Word,
    /// .* (Alt+X, or Alt+R as VS Code).
    Regex,
}

/// The "in" and "files" fields; the text is the box's query.
#[derive(Debug, Default)]
pub(crate) struct FindFields {
    pub(crate) field: Field,
    pub(crate) in_text: String,
    pub(crate) in_caret: usize,
    pub(crate) files: String,
    pub(crate) files_caret: usize,
    pub(crate) options: bb_search::Options,
    pub(crate) scope: Scope,
    /// Each place's folder, by its item's `line`.
    dirs: Vec<PathBuf>,
}

impl QuickOpen {
    /// The Find in Files box over `root`. Its places: `first` (a folder the
    /// Explorer asked for), the workspace, then the active file's project
    /// and folder; [`Self::add_projects`] adds every project after a walk.
    pub(crate) fn find_box(root: &Path, active: Option<&Path>, first: Option<&Path>) -> QuickOpen {
        let mut quick: QuickOpen = QuickOpen::blank();
        quick.find = Some(FindFields::default());
        if let Some(dir) = first {
            quick.add_place(format!("Folder · {}", shown(dir, root)), dir);
        }
        quick.add_place(format!("Workspace · {}", name_of(root)), root);
        if let Some(file) = active.filter(|file: &&Path| file.starts_with(root)) {
            if let Some(project) = project_of(file, root) {
                quick.add_place(project_label(&project, root), &project);
            }
            if let Some(folder) = file.parent() {
                quick.add_place(format!("Folder · {}", shown(folder, root)), folder);
            }
        }
        quick
    }

    /// Every C# project under `root` (its `.csproj` files), after the walk.
    pub(crate) fn add_projects(&mut self, root: &Path, projects: &[PathBuf]) {
        for file in projects {
            if let Some(dir) = file.parent() {
                self.add_place(project_label(dir, root), dir);
            }
        }
    }

    /// A place, once per folder.
    fn add_place(&mut self, label: String, dir: &Path) {
        let Some(fields): Option<&mut FindFields> = self.find.as_mut() else {
            return;
        };
        if fields.dirs.iter().any(|known: &PathBuf| known == dir) {
            return;
        }
        let line: usize = fields.dirs.len();
        fields.dirs.push(dir.to_path_buf());
        self.items.push(Item {
            label: PathBuf::from(label),
            line,
        });
    }

    /// Flip a search toggle.
    pub(crate) fn toggle(&mut self, which: Toggle) {
        if let Some(fields) = self.find.as_mut() {
            let on: &mut bool = match which {
                Toggle::Case => &mut fields.options.case,
                Toggle::Word => &mut fields.options.word,
                Toggle::Regex => &mut fields.options.regex,
            };
            *on = !*on;
        }
    }

    /// A scope button: the list keeps that kind of place; a second press
    /// on the same button shows every place again.
    pub(crate) fn set_scope(&mut self, scope: Scope) {
        if let Some(fields) = self.find.as_mut() {
            fields.scope = if fields.scope == scope {
                Scope::All
            } else {
                scope
            };
            self.selected = 0;
        }
    }

    /// Tab / Shift+Tab: the next or previous field.
    pub(crate) fn next_field(&mut self, back: bool) {
        if let Some(fields) = self.find.as_mut() {
            fields.field = fields.field.next(back);
        }
    }

    /// The field keys edit: the query, or a Find in Files field.
    pub(super) fn active_field(&mut self) -> (&mut String, &mut usize) {
        match self.find.as_mut() {
            Some(FindFields {
                field: Field::In,
                in_text,
                in_caret,
                ..
            }) => (in_text, in_caret),
            Some(FindFields {
                field: Field::Files,
                files,
                files_caret,
                ..
            }) => (files, files_caret),
            Some(FindFields {
                field: Field::Text, ..
            })
            | None => (&mut self.query, &mut self.caret),
        }
    }

    /// Enter in Find in Files: the text and the selected place.
    pub(super) fn find_ask(&self, fields: &FindFields) -> Option<FindAsk> {
        if self.query.is_empty() {
            return None;
        }
        let matches: Vec<&Path> = self.matches();
        let label: &Path = matches.get(self.selected)?;
        let item: &Item = self
            .items
            .iter()
            .find(|item: &&Item| std::ptr::eq(item.label.as_path(), label))?;
        Some(FindAsk {
            text: self.query.clone(),
            dir: fields.dirs.get(item.line)?.clone(),
            place: label.display().to_string(),
            files: fields.files.clone(),
            options: fields.options,
        })
    }
}

/// The nearest folder above `file`, inside `root`, holding a `.csproj`.
fn project_of(file: &Path, root: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .take_while(|dir: &&Path| dir.starts_with(root))
        .find(|dir: &&Path| {
            std::fs::read_dir(dir).is_ok_and(|entries: std::fs::ReadDir| {
                entries.flatten().any(|entry: std::fs::DirEntry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|e: &std::ffi::OsStr| e == "csproj")
                })
            })
        })
        .map(Path::to_path_buf)
}
