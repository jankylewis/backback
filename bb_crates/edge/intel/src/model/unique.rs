//! One guard for the "server repeats an answer" family: Roslyn lists the
//! declaration twice in `references` (observed on the harness). A repeated
//! location shows twice in a list; a repeated edit applied twice corrupts the
//! file. Every list the API returns passes through here.

use std::collections::HashSet;
use std::hash::Hash;

use super::results::{FileChange, TextChange};
use super::DocUri;

/// `items` without repeats, first appearance kept, order kept.
pub(crate) fn unique<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen: HashSet<T> = HashSet::with_capacity(items.len());
    items
        .into_iter()
        .filter(|item: &T| seen.insert(item.clone()))
        .collect()
}

/// Drop repeated edits inside each document. When the plan has only edits
/// (no create, rename or delete, whose order matters), merge the edits of one
/// document that came in separate steps.
pub(crate) fn unique_changes(plan: Vec<FileChange>) -> Vec<FileChange> {
    let only_edits: bool = plan
        .iter()
        .all(|c: &FileChange| matches!(c, FileChange::Edit { .. }));
    let mut out: Vec<FileChange> = Vec::with_capacity(plan.len());
    for step in plan {
        match step {
            FileChange::Edit { uri, changes } => push_edit(&mut out, only_edits, uri, changes),
            FileChange::Create(_) | FileChange::Rename { .. } | FileChange::Delete(_) => {
                out.push(step);
            }
        }
    }
    out.into_iter()
        .map(|step: FileChange| match step {
            FileChange::Edit { uri, changes } => FileChange::Edit {
                uri,
                changes: unique(changes),
            },
            FileChange::Create(_) | FileChange::Rename { .. } | FileChange::Delete(_) => step,
        })
        .collect()
}

fn push_edit(out: &mut Vec<FileChange>, merge: bool, uri: DocUri, changes: Vec<TextChange>) {
    let earlier: Option<&mut Vec<TextChange>> = if merge {
        out.iter_mut().find_map(|c: &mut FileChange| match c {
            FileChange::Edit {
                uri: u,
                changes: list,
            } if *u == uri => Some(list),
            FileChange::Edit { .. }
            | FileChange::Create(_)
            | FileChange::Rename { .. }
            | FileChange::Delete(_) => None,
        })
    } else {
        None
    };
    match earlier {
        Some(list) => list.extend(changes),
        None => out.push(FileChange::Edit { uri, changes }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Span;
    use bb_text::LspPoint;

    fn change(col: u32) -> TextChange {
        let at: LspPoint = LspPoint {
            line: 0,
            character: col,
        };
        TextChange {
            span: Span { start: at, end: at },
            new_text: "x".to_owned(),
        }
    }

    fn edit(name: &str, changes: Vec<TextChange>) -> FileChange {
        FileChange::Edit {
            uri: DocUri::Virtual(name.to_owned()),
            changes,
        }
    }

    #[test]
    fn repeats_are_dropped_and_order_is_kept() {
        assert_eq!(unique(vec![3, 1, 3, 2, 1]), vec![3, 1, 2]);
    }

    #[test]
    fn a_document_edited_in_two_steps_gets_one_step_without_repeats() {
        let plan: Vec<FileChange> = vec![
            edit("a", vec![change(1), change(1)]),
            edit("b", vec![change(2)]),
            edit("a", vec![change(1), change(5)]),
        ];
        assert_eq!(
            unique_changes(plan),
            vec![
                edit("a", vec![change(1), change(5)]),
                edit("b", vec![change(2)])
            ]
        );
    }

    #[test]
    fn steps_around_a_file_operation_keep_their_order() {
        let created: FileChange = FileChange::Create(DocUri::Virtual("a".to_owned()));
        let plan: Vec<FileChange> = vec![
            edit("a", vec![change(1)]),
            created.clone(),
            edit("a", vec![change(1), change(1)]),
        ];
        assert_eq!(
            unique_changes(plan),
            vec![
                edit("a", vec![change(1)]),
                created,
                edit("a", vec![change(1)])
            ]
        );
    }
}
