//! Drawing code intelligence: popups, panels, squiggles.

pub(crate) mod bb_popups;
pub(crate) mod marks;
pub(crate) mod panels;

use std::time::Instant;

use crate::intel::model::{IntelModel, TraceRow, Undrawn};

/// Count a frame. A completion answer applied since the last frame is drawn
/// in this one: record how many frames and how long it waited.
pub(crate) fn note_frame(model: &mut IntelModel) {
    model.frame = model.frame.saturating_add(1);
    if model.completion.is_none() {
        model.undrawn = None;
        return;
    }
    let Some(undrawn): Option<Undrawn> = model.undrawn.take() else {
        return;
    };
    let row: TraceRow = TraceRow {
        queued: undrawn.stages.queued(),
        server: undrawn.stages.server(),
        hop: undrawn
            .applied
            .saturating_duration_since(undrawn.stages.answered),
        frames: model.frame.saturating_sub(undrawn.frame).saturating_sub(1),
        to_paint: Instant::now().saturating_duration_since(undrawn.stages.answered),
    };
    eprintln!(
        "intel_trace kind=completion queued_us={} server_us={} hop_us={} frames={} to_paint_us={}",
        row.queued.as_micros(),
        row.server.as_micros(),
        row.hop.as_micros(),
        row.frames,
        row.to_paint.as_micros(),
    );
    model.trace.push(row);
}
