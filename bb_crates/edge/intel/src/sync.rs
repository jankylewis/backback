//! Document sync: `didOpen`, incremental `didChange` from the mirror,
//! full resync, `didClose`. The version of a document only goes up.

use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    VersionedTextDocumentIdentifier,
};

use crate::model::bb_doc::{full_text, DocVersion, Edit, OpenDoc};
use crate::model::results::DiagnosticsReport;
use crate::model::DocPath;
use crate::{IntelError, Session};

impl Session {
    /// Open `path` with the text on disk; returns its first diagnostics.
    ///
    /// # Errors
    /// The file cannot be read, or see [`Session::open_text`].
    pub async fn open(&mut self, path: &DocPath) -> Result<DiagnosticsReport, IntelError> {
        let text: String =
            std::fs::read_to_string(path.as_path()).map_err(|source: std::io::Error| {
                IntelError::Read {
                    path: path.as_path().to_path_buf(),
                    source,
                }
            })?;
        self.open_text(path, &text).await
    }

    /// Open `path` with `text` (`didOpen`, version 1), then pull its
    /// diagnostics and return them.
    ///
    /// The pull is the warm-up: Roslyn answers `completion` with `null` when
    /// it is the first request after `didOpen`, and keeps doing so until any
    /// other request has run. When the pull fails the document is closed
    /// again, so a failed open leaves nothing open.
    ///
    /// # Errors
    /// Already open, a bad path, a closed connection, or the pull fails.
    pub async fn open_text(
        &mut self,
        path: &DocPath,
        text: &str,
    ) -> Result<DiagnosticsReport, IntelError> {
        if self.docs.contains_key(path) {
            return Err(IntelError::AlreadyOpen(path.as_path().to_path_buf()));
        }
        let doc: OpenDoc = OpenDoc::new(text);
        let params: DidOpenTextDocumentParams = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: path.to_url()?,
                language_id: "csharp".to_owned(),
                version: doc.version.raw(),
                text: text.to_owned(),
            },
        };
        self.client.notify("textDocument/didOpen", params).await?;
        self.docs.insert(path.clone(), doc);
        match self.diagnostics(path, self.request_timeout).await {
            Ok(found) => Ok(found),
            Err(failed) => {
                // The pull already failed; that error is the one to report.
                let _closed: Result<(), IntelError> = self.close(path).await;
                Err(failed)
            }
        }
    }

    /// Apply `edits` in order and send them as one incremental `didChange`.
    /// Returns the new version. Nothing changes when any edit is out of range.
    ///
    /// # Errors
    /// Not open, an edit out of range, version overflow, or a closed connection.
    pub async fn change(
        &mut self,
        path: &DocPath,
        edits: &[Edit],
    ) -> Result<DocVersion, IntelError> {
        let doc: &mut OpenDoc = self.doc_mut(path)?;
        let (version, events): (DocVersion, Vec<TextDocumentContentChangeEvent>) = doc
            .apply(edits)?
            .ok_or_else(|| IntelError::VersionOverflow(path.as_path().to_path_buf()))?;
        self.send_change(path, version, events).await?;
        Ok(version)
    }

    /// Send the whole mirror text as a full `didChange` (a resync).
    ///
    /// # Errors
    /// Not open, version overflow, or a closed connection.
    pub async fn resync(&mut self, path: &DocPath) -> Result<DocVersion, IntelError> {
        let doc: &mut OpenDoc = self.doc_mut(path)?;
        let version: DocVersion = doc
            .version
            .next()
            .ok_or_else(|| IntelError::VersionOverflow(path.as_path().to_path_buf()))?;
        doc.version = version;
        let event: TextDocumentContentChangeEvent = full_text(&doc.buffer);
        self.send_change(path, version, vec![event]).await?;
        Ok(version)
    }

    /// Close `path` (`didClose`) and drop its mirror and diagnostics.
    ///
    /// # Errors
    /// Not open, or a closed connection.
    pub async fn close(&mut self, path: &DocPath) -> Result<(), IntelError> {
        self.doc(path)?;
        let params: DidCloseTextDocumentParams = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: path.to_url()?,
            },
        };
        self.client.notify("textDocument/didClose", params).await?;
        self.docs.remove(path);
        self.store.remove(path)
    }

    fn doc_mut(&mut self, path: &DocPath) -> Result<&mut OpenDoc, IntelError> {
        self.docs
            .get_mut(path)
            .ok_or_else(|| IntelError::NotOpen(path.as_path().to_path_buf()))
    }

    async fn send_change(
        &self,
        path: &DocPath,
        version: DocVersion,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), IntelError> {
        let params: DidChangeTextDocumentParams = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: path.to_url()?,
                version: version.raw(),
            },
            content_changes,
        };
        Ok(self.client.notify("textDocument/didChange", params).await?)
    }
}
