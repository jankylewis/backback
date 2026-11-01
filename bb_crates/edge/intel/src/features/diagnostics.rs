//! Diagnostics, collected per document.
//!
//! Roslyn answers pull requests (`textDocument/diagnostic`) and, on
//! roslyn-language-server 5.x, pushes no `publishDiagnostics`. Both paths feed
//! one store, so [`Session::collected`] shows the last report either way.

use std::time::Duration;

use lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    TextDocumentIdentifier,
};

use crate::model::results::Diagnostic;
use crate::model::{DocPath, Span};
use crate::{IntelError, Session};

impl Session {
    /// Ask the server for the diagnostics of an open document, keep them, and
    /// return them.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn diagnostics(
        &self,
        path: &DocPath,
        deadline: Duration,
    ) -> Result<Vec<Diagnostic>, IntelError> {
        self.doc(path)?;
        let params: DocumentDiagnosticParams = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier {
                uri: path.to_url()?,
            },
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let answer: DocumentDiagnosticReportResult = self
            .raw("textDocument/diagnostic", params, deadline)
            .await?;
        if let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) = answer
        {
            if let Ok(mut map) = self.store.lock() {
                map.insert(path.clone(), full.full_document_diagnostic_report.items);
            }
        }
        Ok(self.collected(path))
    }

    /// The diagnostics last reported for `path` (pushed or pulled).
    #[must_use]
    pub fn collected(&self, path: &DocPath) -> Vec<Diagnostic> {
        self.store.lock().map_or_else(
            |_: std::sync::PoisonError<_>| Vec::new(),
            |map: std::sync::MutexGuard<'_, _>| {
                map.get(path)
                    .map(|list: &Vec<lsp_types::Diagnostic>| {
                        list.iter().map(Diagnostic::from_lsp).collect()
                    })
                    .unwrap_or_default()
            },
        )
    }

    /// The stored LSP diagnostics that touch `span`, for code action context.
    pub(crate) fn raw_diagnostics_in(
        &self,
        path: &DocPath,
        span: Span,
    ) -> Vec<lsp_types::Diagnostic> {
        let wanted: lsp_types::Range = span.to_lsp();
        self.store.lock().map_or_else(
            |_: std::sync::PoisonError<_>| Vec::new(),
            |map: std::sync::MutexGuard<'_, _>| {
                map.get(path)
                    .map(|list: &Vec<lsp_types::Diagnostic>| {
                        list.iter()
                            .filter(|d: &&lsp_types::Diagnostic| overlaps(d.range, wanted))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default()
            },
        )
    }
}

fn overlaps(a: lsp_types::Range, b: lsp_types::Range) -> bool {
    a.start <= b.end && b.start <= a.end
}
