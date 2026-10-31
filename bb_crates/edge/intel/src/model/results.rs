//! Feature answers, converted from the LSP shapes into plain values.

use lsp_types::{
    CompletionItemKind, DocumentChangeOperation, DocumentChanges, NumberOrString, OneOf,
    ResourceOp, TextDocumentEdit, WorkspaceEdit,
};

use super::{DocUri, Span};

/// How bad a diagnostic is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The code does not compile.
    Error,
    /// A warning.
    Warning,
    /// Information.
    Information,
    /// A hint (often faded in the editor).
    Hint,
}

impl Severity {
    fn from_lsp(s: lsp_types::DiagnosticSeverity) -> Option<Self> {
        match s {
            lsp_types::DiagnosticSeverity::ERROR => Some(Self::Error),
            lsp_types::DiagnosticSeverity::WARNING => Some(Self::Warning),
            lsp_types::DiagnosticSeverity::INFORMATION => Some(Self::Information),
            lsp_types::DiagnosticSeverity::HINT => Some(Self::Hint),
            _ => None,
        }
    }
}

/// One problem the server found in a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Where.
    pub span: Span,
    /// How bad; `None` when the server sent none or an unknown value.
    pub severity: Option<Severity>,
    /// The rule id, such as `CS1002`.
    pub code: Option<String>,
    /// The text.
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn from_lsp(d: &lsp_types::Diagnostic) -> Self {
        Self {
            span: Span::from_lsp(d.range),
            severity: d.severity.and_then(Severity::from_lsp),
            code: d.code.as_ref().map(|c: &NumberOrString| match c {
                NumberOrString::Number(n) => n.to_string(),
                NumberOrString::String(s) => s.clone(),
            }),
            message: d.message.clone(),
        }
    }
}

/// One completion item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionEntry {
    /// What the list shows.
    pub label: String,
    /// The kind (method, class, ...), if given.
    pub kind: Option<CompletionItemKind>,
    /// Extra detail, if given.
    pub detail: Option<String>,
}

/// What hover shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverInfo {
    /// The text, parts joined by a blank line.
    pub text: String,
    /// The range the hover is about, if given.
    pub span: Option<Span>,
}

/// One replacement inside a document.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextChange {
    /// What is replaced.
    pub span: Span,
    /// The new text.
    pub new_text: String,
}

/// One step of a workspace edit. Returned, never applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileChange {
    /// Replacements in one document.
    Edit {
        /// The document.
        uri: DocUri,
        /// The replacements, in server order.
        changes: Vec<TextChange>,
    },
    /// Create a file.
    Create(DocUri),
    /// Rename a file.
    Rename {
        /// Old name.
        from: DocUri,
        /// New name.
        to: DocUri,
    },
    /// Delete a file.
    Delete(DocUri),
}

impl FileChange {
    /// All steps of `edit`: `documentChanges` when present (the LSP rule),
    /// else `changes` sorted by URI.
    pub(crate) fn all_from(edit: WorkspaceEdit) -> Vec<Self> {
        super::unique::unique_changes(Self::steps(edit))
    }

    fn steps(edit: WorkspaceEdit) -> Vec<Self> {
        if let Some(steps) = edit.document_changes {
            return match steps {
                DocumentChanges::Edits(edits) => edits.iter().map(Self::from_edit).collect(),
                DocumentChanges::Operations(ops) => ops.into_iter().map(Self::from_op).collect(),
            };
        }
        let mut found: Vec<Self> = edit
            .changes
            .unwrap_or_default()
            .into_iter()
            .map(
                |(url, edits): (url::Url, Vec<lsp_types::TextEdit>)| Self::Edit {
                    uri: DocUri::from_url(&url),
                    changes: edits.iter().map(text_change).collect(),
                },
            )
            .collect();
        found.sort_by_key(Self::sort_key);
        found
    }

    fn from_edit(edit: &TextDocumentEdit) -> Self {
        let changes: Vec<TextChange> = edit
            .edits
            .iter()
            .map(
                |e: &OneOf<lsp_types::TextEdit, lsp_types::AnnotatedTextEdit>| match e {
                    OneOf::Left(plain) => text_change(plain),
                    OneOf::Right(annotated) => text_change(&annotated.text_edit),
                },
            )
            .collect();
        Self::Edit {
            uri: DocUri::from_url(&edit.text_document.uri),
            changes,
        }
    }

    fn from_op(op: DocumentChangeOperation) -> Self {
        match op {
            DocumentChangeOperation::Edit(edit) => Self::from_edit(&edit),
            DocumentChangeOperation::Op(ResourceOp::Create(c)) => {
                Self::Create(DocUri::from_url(&c.uri))
            }
            DocumentChangeOperation::Op(ResourceOp::Rename(r)) => Self::Rename {
                from: DocUri::from_url(&r.old_uri),
                to: DocUri::from_url(&r.new_uri),
            },
            DocumentChangeOperation::Op(ResourceOp::Delete(d)) => {
                Self::Delete(DocUri::from_url(&d.uri))
            }
        }
    }

    /// The document the step is about (the old name for a rename).
    #[must_use]
    pub fn uri(&self) -> &DocUri {
        match self {
            Self::Edit { uri, .. } | Self::Create(uri) | Self::Delete(uri) => uri,
            Self::Rename { from, .. } => from,
        }
    }

    fn sort_key(&self) -> DocUri {
        self.uri().clone()
    }
}

pub(crate) fn text_change(e: &lsp_types::TextEdit) -> TextChange {
    TextChange {
        span: Span::from_lsp(e.range),
        new_text: e.new_text.clone(),
    }
}

/// One code action, listed only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionItem {
    /// What the menu shows.
    pub title: String,
    /// The kind (`quickfix`, `refactor.extract`, ...), if given.
    pub kind: Option<String>,
}
