//! The editor features. Each is one typed call with its own deadline, asked
//! only for an open document so positions match the mirror the server holds.

mod bb_nav;
mod diagnostics;
mod edits;

use std::time::Duration;

use bb_text::LspPoint;
use lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, Hover, HoverContents, HoverParams,
    MarkedString, TextDocumentIdentifier, TextDocumentPositionParams,
};

use crate::model::results::{CompletionEntry, HoverInfo};
use crate::model::{point_to_lsp, DocPath, Span};
use crate::{IntelError, Session};

impl Session {
    /// Completion items at `at`.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn completion(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<CompletionEntry>, IntelError> {
        let params: CompletionParams = CompletionParams {
            text_document_position: self.position(path, at)?,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
            context: None,
        };
        let answer: Option<CompletionResponse> = self
            .raw("textDocument/completion", params, deadline)
            .await?;
        let items: Vec<CompletionItem> = match answer {
            Some(CompletionResponse::Array(items)) => items,
            Some(CompletionResponse::List(list)) => list.items,
            None => Vec::new(),
        };
        Ok(items
            .into_iter()
            .map(|i: CompletionItem| CompletionEntry {
                label: i.label,
                kind: i.kind,
                detail: i.detail,
            })
            .collect())
    }

    /// Hover text at `at`; `None` when the server has nothing to show.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn hover(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Option<HoverInfo>, IntelError> {
        let params: HoverParams = HoverParams {
            text_document_position_params: self.position(path, at)?,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
        };
        let answer: Option<Hover> = self.raw("textDocument/hover", params, deadline).await?;
        Ok(answer.map(|h: Hover| HoverInfo {
            text: hover_text(h.contents),
            span: h.range.map(Span::from_lsp),
        }))
    }

    /// The document + position params for an open document.
    pub(crate) fn position(
        &self,
        path: &DocPath,
        at: LspPoint,
    ) -> Result<TextDocumentPositionParams, IntelError> {
        self.doc(path)?;
        Ok(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: path.to_url()?,
            },
            position: point_to_lsp(at),
        })
    }
}

fn hover_text(contents: HoverContents) -> String {
    let marked: fn(MarkedString) -> String = |m: MarkedString| match m {
        MarkedString::String(s) => s,
        MarkedString::LanguageString(l) => l.value,
    };
    match contents {
        HoverContents::Scalar(m) => marked(m),
        HoverContents::Array(parts) => parts
            .into_iter()
            .map(marked)
            .collect::<Vec<String>>()
            .join("\n\n"),
        HoverContents::Markup(m) => m.value,
    }
}
