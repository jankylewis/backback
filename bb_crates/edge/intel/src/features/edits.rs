//! Rename and format (the edits are returned, never applied) and code
//! actions (listed).

use std::time::Duration;

use bb_text::LspPoint;
use lsp_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, DocumentFormattingParams,
    FormattingOptions, RenameParams, TextEdit, WorkspaceEdit,
};

use crate::model::results::{text_change, ActionItem, FileChange, TextChange};
use crate::model::{DocPath, Span};
use crate::{IntelError, Session};

impl Session {
    /// The edit that renames the symbol at `at` to `new_name`. Nothing is
    /// applied; an empty list means the server offered no edit.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn rename(
        &self,
        path: &DocPath,
        at: LspPoint,
        new_name: &str,
        deadline: Duration,
    ) -> Result<Vec<FileChange>, IntelError> {
        let params: RenameParams = RenameParams {
            text_document_position: self.position(path, at)?,
            new_name: new_name.to_owned(),
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
        };
        let answer: Option<WorkspaceEdit> =
            self.raw("textDocument/rename", params, deadline).await?;
        Ok(answer.map(FileChange::all_from).unwrap_or_default())
    }

    /// The edits that format the whole document (4 spaces). Nothing is
    /// applied; an empty list means it is already formatted.
    ///
    /// # Errors
    /// Not open; [`IntelError::Formatter`] when the server's formatter
    /// fails; else the request fails or passes `deadline`.
    pub async fn format(
        &self,
        path: &DocPath,
        deadline: Duration,
    ) -> Result<Vec<TextChange>, IntelError> {
        self.doc(path)?;
        let params: DocumentFormattingParams = DocumentFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: path.to_url()?,
            },
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..FormattingOptions::default()
            },
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
        };
        let answer: Result<Option<Vec<TextEdit>>, IntelError> =
            self.raw("textDocument/formatting", params, deadline).await;
        match answer {
            Ok(edits) => Ok(edits.unwrap_or_default().iter().map(text_change).collect()),
            Err(IntelError::Lsp(bb_lsp::LspError::Server { code, message })) => {
                Err(IntelError::Formatter {
                    path: path.as_path().to_path_buf(),
                    code,
                    message,
                })
            }
            Err(other) => Err(other),
        }
    }

    /// The code actions offered for `span`, with the diagnostics last seen
    /// there as context.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn code_actions(
        &self,
        path: &DocPath,
        span: Span,
        deadline: Duration,
    ) -> Result<Vec<ActionItem>, IntelError> {
        let doc_id: lsp_types::TextDocumentIdentifier =
            self.position(path, span.start)?.text_document;
        let params: CodeActionParams = CodeActionParams {
            text_document: doc_id,
            range: span.to_lsp(),
            context: CodeActionContext {
                diagnostics: self.raw_diagnostics_in(path, span)?,
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let answer: Option<Vec<CodeActionOrCommand>> = self
            .raw("textDocument/codeAction", params, deadline)
            .await?;
        Ok(answer
            .unwrap_or_default()
            .into_iter()
            .map(|a: CodeActionOrCommand| match a {
                CodeActionOrCommand::Command(c) => ActionItem {
                    title: c.title,
                    kind: None,
                },
                CodeActionOrCommand::CodeAction(c) => ActionItem {
                    title: c.title,
                    kind: c
                        .kind
                        .map(|k: lsp_types::CodeActionKind| k.as_str().to_owned()),
                },
            })
            .collect())
    }
}
