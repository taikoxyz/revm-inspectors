//! Gwyneth integration helpers for tracing input materialization and selector extraction.
//! This module owns xchain-facing error mapping and fail-closed fallbacks.

use revm::{
    interpreter::CallInputReadError,
};

/// Recoverable tracing issues captured while preserving runtime behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceRecoverableError {
    /// Call input could not be materialized from shared memory.
    CallInputRead(CallInputReadError),
    /// Trace stack bookkeeping drifted from expected call lifecycle sequencing.
    TraceStackInvariant(&'static str),
}

/// Reads call input bytes and falls back to empty bytes on read failures.
#[inline]
#[cfg(feature = "js-tracer")]
#[allow(dead_code)]
pub(crate) fn input_bytes<CTX: revm::context_interface::ContextTr>(
    context: &mut CTX,
    inputs: &revm::interpreter::CallInputs,
) -> revm::primitives::Bytes {
    super::measurement_input::try_input_bytes(context, inputs).unwrap_or_default()
}
