//! Mirror of `gwyneth-types` call-input prefix helpers for inspector-only parity checks.
//! Keep in sync with canonical owner via `scripts/check_phase68_selector_prefix_owner.sh`.
//
// BEGIN_CALL_INPUT_PREFIX_TABLE
// 0x00000000	selector_prefix	optional	bytes[0..4] selector prefix
// 0x00000000	inner_selector_prefix	optional	bytes[4..8] inner selector prefix
// END_CALL_INPUT_PREFIX_TABLE

/// Reads a fixed-size prefix at `start` from `input`.
#[inline]
#[allow(dead_code)]
pub(crate) fn read_array<const N: usize>(input: &[u8], start: usize) -> Option<[u8; N]> {
    let end = start.checked_add(N)?;
    let slice = input.get(start..end)?;
    let mut out = [0u8; N];
    out.copy_from_slice(slice);
    Some(out)
}

/// Returns `(input[0..4], input[4..8])` when present.
#[inline]
#[allow(dead_code)]
pub(crate) fn selector_prefixes(input: &[u8]) -> (Option<[u8; 4]>, Option<[u8; 4]>) {
    let selector = read_array::<4>(input, 0);
    let inner = read_array::<4>(input, 4);
    (selector, inner)
}

/// Public mirror accessor for external parity tests.
#[inline]
pub fn selector_prefixes_for_tests(input: &[u8]) -> (Option<[u8; 4]>, Option<[u8; 4]>) {
    selector_prefixes(input)
}
