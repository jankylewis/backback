//! Call hierarchy (BBEOD24): who calls the symbol at a point. Two steps, as
//! in LSP: prepare names the symbol, then each item asks for its callers.

use std::time::Duration;

use bb_text::LspPoint;
use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyPrepareParams, PartialResultParams, Range, WorkDoneProgressParams,
};

use crate::model::unique::unique;
use crate::model::{DocPath, DocUri, Location, Span};
use crate::{IntelError, Session};

/// A symbol the server can list the callers of.
#[derive(Clone, Debug, PartialEq)]
pub struct CallItem {
    /// Its name, as the server shows it.
    pub name: String,
    /// Where its name is declared.
    pub location: Location,
    /// The item as the server sent it: [`Session::incoming_calls`] sends it
    /// back whole, with the `data` the server finds the symbol by.
    raw: CallHierarchyItem,
}

impl CallItem {
    fn from_lsp(raw: CallHierarchyItem) -> Self {
        Self {
            name: raw.name.clone(),
            location: Location {
                uri: DocUri::from_url(&raw.uri),
                span: Span::from_lsp(raw.selection_range),
            },
            raw,
        }
    }
}

/// One caller of a [`CallItem`] and where it calls it.
#[derive(Clone, Debug, PartialEq)]
pub struct IncomingCall {
    /// The calling symbol; [`Session::incoming_calls`] lists its own callers.
    pub from: CallItem,
    /// Each call site (LSP `fromRanges`), in the caller's document.
    pub sites: Vec<Location>,
}

impl IncomingCall {
    fn from_lsp(call: CallHierarchyIncomingCall) -> Self {
        let uri: DocUri = DocUri::from_url(&call.from.uri);
        let sites: Vec<Location> = call
            .from_ranges
            .iter()
            .map(|range: &Range| Location {
                uri: uri.clone(),
                span: Span::from_lsp(*range),
            })
            .collect();
        Self {
            from: CallItem::from_lsp(call.from),
            sites: unique(sites),
        }
    }
}

impl Session {
    /// The symbols at `at` whose callers can be listed; empty when there is
    /// none there.
    ///
    /// # Errors
    /// Not open, or the request fails or passes `deadline`.
    pub async fn prepare_call_hierarchy(
        &self,
        path: &DocPath,
        at: LspPoint,
        deadline: Duration,
    ) -> Result<Vec<CallItem>, IntelError> {
        let params: CallHierarchyPrepareParams = CallHierarchyPrepareParams {
            text_document_position_params: self.position(path, at)?,
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        let answer: Option<Vec<CallHierarchyItem>> = self
            .raw("textDocument/prepareCallHierarchy", params, deadline)
            .await?;
        Ok(answer
            .unwrap_or_default()
            .into_iter()
            .map(CallItem::from_lsp)
            .collect())
    }

    /// Who calls `item`: each caller with its call sites.
    ///
    /// # Errors
    /// The request fails or passes `deadline`.
    pub async fn incoming_calls(
        &self,
        item: &CallItem,
        deadline: Duration,
    ) -> Result<Vec<IncomingCall>, IntelError> {
        let params: CallHierarchyIncomingCallsParams = CallHierarchyIncomingCallsParams {
            item: item.raw.clone(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let answer: Option<Vec<CallHierarchyIncomingCall>> = self
            .raw("callHierarchy/incomingCalls", params, deadline)
            .await?;
        Ok(answer
            .unwrap_or_default()
            .into_iter()
            .map(IncomingCall::from_lsp)
            .collect())
    }
}
