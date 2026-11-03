//! Navigation: go to definition, implementation and type definition;
//! find references.

use std::time::Duration;

use bb_text::LspPoint;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, LocationLink, ReferenceParams};

use crate::model::unique::unique;
use crate::model::{DocPath, DocUri, Location, Span};
use crate::{IntelError, Session};

impl Session {
    /// Where the symbol at `at` is defined.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn definition(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<Location>, IntelError> {
        self.goto("textDocument/definition", path, at, deadline)
            .await
    }

    /// Where the symbol at `at` is implemented: the types and members that
    /// implement or override it (BBEOD22).
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn implementation(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<Location>, IntelError> {
        self.goto("textDocument/implementation", path, at, deadline)
            .await
    }

    /// Where the type of the symbol at `at` is defined (BBEOD22).
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn type_definition(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<Location>, IntelError> {
        self.goto("textDocument/typeDefinition", path, at, deadline)
            .await
    }

    /// One of the go-to requests: they share their parameters and their
    /// answer (one location, a list, or links).
    async fn goto(
        &self,
        method: &str,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<Location>, IntelError> {
        let params: GotoDefinitionParams = GotoDefinitionParams {
            text_document_position_params: self.position(path, at)?,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
        };
        let answer: Option<GotoDefinitionResponse> = self.raw(method, params, deadline).await?;
        Ok(unique(match answer {
            Some(GotoDefinitionResponse::Scalar(one)) => vec![Location::from_lsp(&one)],
            Some(GotoDefinitionResponse::Array(many)) => {
                many.iter().map(Location::from_lsp).collect()
            }
            Some(GotoDefinitionResponse::Link(links)) => links
                .iter()
                .map(|l: &LocationLink| Location {
                    uri: DocUri::from_url(&l.target_uri),
                    span: Span::from_lsp(l.target_selection_range),
                })
                .collect(),
            None => Vec::new(),
        }))
    }

    /// Every use of the symbol at `at`, with its declaration.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn references(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<Location>, IntelError> {
        let params: ReferenceParams = ReferenceParams {
            text_document_position: self.position(path, at)?,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: lsp_types::PartialResultParams::default(),
            context: lsp_types::ReferenceContext {
                include_declaration: true,
            },
        };
        let answer: Option<Vec<lsp_types::Location>> = self
            .raw("textDocument/references", params, deadline)
            .await?;
        Ok(unique(
            answer
                .unwrap_or_default()
                .iter()
                .map(Location::from_lsp)
                .collect(),
        ))
    }
}
