//! Measurement/diagnostics input materialization kernel shared by tracing integrations.

use revm::{
    context_interface::ContextTr,
    interpreter::{CallInputReadError, CallInputs},
    primitives::Bytes,
};

/// Attempts to read call input bytes without changing runtime behavior.
#[inline]
#[allow(dead_code)]
pub(crate) fn try_input_bytes<CTX: ContextTr>(
    context: &mut CTX,
    inputs: &CallInputs,
) -> Result<Bytes, CallInputReadError> {
    Ok(inputs.input.bytes(context))
}
