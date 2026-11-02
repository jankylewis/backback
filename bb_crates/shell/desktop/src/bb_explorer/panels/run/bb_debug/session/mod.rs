//! The debug session: a thread that owns netcoredbg, and how a stop is read.

mod bb_worker;
pub(crate) mod control;
mod stops;
#[cfg(test)]
mod tests;

pub(crate) use bb_worker::{start, Command, Frame, Note, Start, StepKind, Var};
