//! Diagnostics, collected per document and tied to its version.
//!
//! Roslyn answers pull requests (`textDocument/diagnostic`) and, on
//! roslyn-language-server 5.x, pushes no `publishDiagnostics`. Both paths feed
//! one versioned store; a report for an older version never replaces a newer
//! one, so [`Session::collected`] shows the newest report either way.

use std::time::Duration;

use bb_lsp::roslyn;
use bb_lsp::Incoming;
use lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    TextDocumentIdentifier,
};

use crate::model::bb_doc::DocVersion;
use crate::model::results::{Diagnostic, DiagnosticsReport};
use crate::model::store::{Entry, Recorded};
use crate::model::{DocPath, Span};
use crate::{IntelError, Session};

impl Session {
    /// Ask the server for the diagnostics of an open document at its current
    /// version, keep them, and return that answer (never an older list).
    ///
    /// # Errors
    /// Not open, the request fails or passes `deadline`, the store is
    /// poisoned, or the server sends no full list and none is stored.
    pub async fn diagnostics(
        &self,
        path: &DocPath,
        deadline: Duration,
    ) -> Result<DiagnosticsReport, IntelError> {
        // `&self` is held across the request, so no edit can land meanwhile:
        // the answer is for this version.
        let version: DocVersion = self.doc(path)?.version;
        let stored: Option<Entry> = self.store.entry(path)?;
        let params: DocumentDiagnosticParams = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier {
                uri: path.to_url()?,
            },
            identifier: None,
            // Always ask for the full list: Roslyn answers "unchanged" after
            // an edit it has not analysed yet (BBEOD03-DIAGVER readback).
            previous_result_id: None,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let answer: DocumentDiagnosticReportResult = self
            .raw("textDocument/diagnostic", params, deadline)
            .await?;
        let entry: Entry = entry_of(answer, stored, version, path)?;
        let list: Vec<Diagnostic> = entry.list.iter().map(Diagnostic::from_lsp).collect();
        // A newer stored report cannot exist (no edit landed); the answer
        // stands either way.
        let _recorded: Recorded = self.store.record(path, entry)?;
        Ok(DiagnosticsReport { version, list })
    }

    /// The newest report stored for `path` (pushed or pulled) that names
    /// its version; `None` when there is none.
    ///
    /// # Errors
    /// [`IntelError::StorePoisoned`].
    pub fn collected(&self, path: &DocPath) -> Result<Option<DiagnosticsReport>, IntelError> {
        Ok(self.store.entry(path)?.and_then(|e: Entry| {
            e.version.map(|version: DocVersion| DiagnosticsReport {
                version,
                list: e.list.iter().map(Diagnostic::from_lsp).collect(),
            })
        }))
    }

    /// The stored LSP diagnostics that touch `span`, for code action context.
    pub(crate) fn raw_diagnostics_in(
        &self,
        path: &DocPath,
        span: Span,
    ) -> Result<Vec<lsp_types::Diagnostic>, IntelError> {
        let wanted: lsp_types::Range = span.to_lsp();
        Ok(self
            .store
            .entry(path)?
            .map(|e: Entry| {
                e.list
                    .into_iter()
                    .filter(|d: &lsp_types::Diagnostic| overlaps(d.range, wanted))
                    .collect()
            })
            .unwrap_or_default())
    }
}

/// The entry a pull answer stands for at `version`.
fn entry_of(
    answer: DocumentDiagnosticReportResult,
    stored: Option<Entry>,
    version: DocVersion,
    path: &DocPath,
) -> Result<Entry, IntelError> {
    Ok(match (answer, stored) {
        (DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)), _) => {
            Entry {
                version: Some(version),
                list: full.full_document_diagnostic_report.items,
            }
        }
        // "Unchanged" holds only for a list stored for this same version; a
        // list of an older version never moves onto newer text.
        (
            DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Unchanged(_)),
            Some(held),
        ) if held.version == Some(version) => held,
        (
            DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Unchanged(_))
            | DocumentDiagnosticReportResult::Partial(_),
            _,
        ) => {
            return Err(IntelError::NoReport(path.as_path().to_path_buf()));
        }
    })
}

fn overlaps(a: lsp_types::Range, b: lsp_types::Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

#[cfg(test)]
mod tests {
    use lsp_types::{
        DocumentDiagnosticReport, DocumentDiagnosticReportResult,
        RelatedUnchangedDocumentDiagnosticReport, UnchangedDocumentDiagnosticReport,
    };

    use super::entry_of;
    use crate::model::bb_doc::DocVersion;
    use crate::model::store::Entry;
    use crate::model::DocPath;
    use crate::IntelError;

    fn unchanged() -> DocumentDiagnosticReportResult {
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Unchanged(
            RelatedUnchangedDocumentDiagnosticReport {
                related_documents: None,
                unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                    result_id: String::from("r1"),
                },
            },
        ))
    }

    fn held(version: i32) -> Entry {
        Entry {
            version: DocVersion::from_wire(version),
            list: vec![lsp_types::Diagnostic::default()],
        }
    }

    /// Live readback: Roslyn answered "unchanged" after an edit, and the
    /// old list showed on the new text, one edit behind.
    #[test]
    fn an_unchanged_answer_keeps_only_the_list_of_that_same_version() {
        let path: DocPath = DocPath::new("/ws/A.cs".into()).unwrap();
        let two: DocVersion = DocVersion::from_wire(2).unwrap();
        let same: Entry = entry_of(unchanged(), Some(held(2)), two, &path).unwrap();
        assert_eq!(same, held(2));
        let older: Result<Entry, IntelError> = entry_of(unchanged(), Some(held(1)), two, &path);
        assert!(matches!(older, Err(IntelError::NoReport(_))), "{older:?}");
        let none: Result<Entry, IntelError> = entry_of(unchanged(), None, two, &path);
        assert!(matches!(none, Err(IntelError::NoReport(_))), "{none:?}");
    }
}

/// Whether a server message means every pulled list may be stale: the
/// `workspace/diagnostic/refresh` request, or a project (re)load line in the
/// log (BB-BUG-019: a restore outside the editor reloads the projects, and
/// the lists pulled before it stayed on screen).
pub(crate) fn refresh_requested(message: &Incoming, text: &str) -> bool {
    let method: &str = match message {
        Incoming::ServerRequest { method, .. } | Incoming::Notification { method, .. } => method,
    };
    method == roslyn::DIAGNOSTIC_REFRESH || roslyn::is_project_reload(text)
}

#[cfg(test)]
mod refresh_tests {
    use super::refresh_requested;
    use bb_lsp::Incoming;

    fn request(method: &str) -> Incoming {
        Incoming::ServerRequest {
            method: method.to_owned(),
            params: None,
        }
    }

    fn log(text: &str) -> (Incoming, String) {
        let message: Incoming = Incoming::Notification {
            method: String::from("window/logMessage"),
            params: Some(serde_json::json!({"type": 3, "message": text})),
        };
        (message, text.to_owned())
    }

    #[test]
    fn the_refresh_request_and_a_reload_line_ask_for_a_refresh_and_the_rest_do_not() {
        assert!(refresh_requested(
            &request("workspace/diagnostic/refresh"),
            ""
        ));
        assert!(!refresh_requested(
            &request("window/workDoneProgress/create"),
            ""
        ));
        let (reload, text): (Incoming, String) =
            log("[LanguageServerProjectSystem] Completed (re)load of 1 project(s) in 00:00:00.65");
        assert!(refresh_requested(&reload, &text));
        let (other, text): (Incoming, String) = log("Restore complete");
        assert!(!refresh_requested(&other, &text));
    }
}
