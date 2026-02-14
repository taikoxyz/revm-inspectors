//! Gwyneth integration helpers for tracing input materialization and selector extraction.
//! This module owns xchain-facing error mapping and fail-closed fallbacks.

use super::measurement_input::try_input_bytes;
use alloy_primitives::Selector;
use revm::{
    context_interface::ContextTr,
    interpreter::{CallInputReadError, CallInputs},
    primitives::Bytes,
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
pub(crate) fn input_bytes<CTX: ContextTr>(context: &mut CTX, inputs: &CallInputs) -> Bytes {
    try_input_bytes(context, inputs).unwrap_or_default()
}

/// Reads call input bytes and returns a recoverable error when fallback is used.
#[inline]
#[allow(dead_code)]
pub(crate) fn fallback_input_bytes<CTX: ContextTr>(
    context: &mut CTX,
    inputs: &CallInputs,
) -> (Bytes, Option<TraceRecoverableError>) {
    match try_input_bytes(context, inputs) {
        Ok(input) => (input, None),
        Err(err) => (Bytes::new(), Some(TraceRecoverableError::CallInputRead(err))),
    }
}

/// Returns selector + calldata-size pair used by the fourbyte inspector.
#[inline]
#[allow(dead_code)]
pub(crate) fn selector_and_calldata_size<CTX: ContextTr>(
    context: &mut CTX,
    inputs: &CallInputs,
) -> Option<(Selector, usize)> {
    let Ok((Some(selector_prefix), _)) = inputs.input.try_selector_prefixes(context) else {
        return None;
    };
    Some((Selector::from(selector_prefix), inputs.input.len().saturating_sub(4)))
}
