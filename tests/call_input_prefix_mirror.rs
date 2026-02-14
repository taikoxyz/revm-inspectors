//! Golden-fixture parity tests for the inspectors call-input prefix mirror.

use revm_inspectors::tracing::integration::call_input_prefix_mirror::selector_prefixes_for_tests;

const FIXTURE: &str = include_str!("../src/tracing/testdata/call_input_prefix_golden.tsv");

fn decode_hex_payload(raw: &str) -> Vec<u8> {
    let trimmed = raw.trim();
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if body.is_empty() {
        return Vec::new();
    }
    assert_eq!(body.len() % 2, 0, "fixture hex payload must have even length");
    body.as_bytes()
        .chunks(2)
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16).expect("invalid fixture hex nibble");
            let lo = (pair[1] as char).to_digit(16).expect("invalid fixture hex nibble");
            ((hi << 4) | lo) as u8
        })
        .collect()
}

fn decode_selector(raw: &str) -> Option<[u8; 4]> {
    let trimmed = raw.trim();
    if trimmed == "none" {
        return None;
    }
    let bytes = decode_hex_payload(trimmed);
    assert_eq!(bytes.len(), 4, "fixture selector must be 4 bytes");
    let mut out = [0u8; 4];
    out.copy_from_slice(&bytes);
    Some(out)
}

#[test]
fn call_input_prefix_mirror_golden_fixture_parity() {
    let mut rows = FIXTURE.lines();
    let header = rows.next().expect("fixture header missing");
    assert_eq!(
        header,
        "case_id\tinput_hex\texpected_selector\texpected_inner",
        "fixture header drift"
    );

    let mut seen = 0usize;
    for row in rows {
        if row.trim().is_empty() || row.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = row.split('\t').collect();
        assert_eq!(cols.len(), 4, "fixture row must have 4 columns: {row}");
        let input = decode_hex_payload(cols[1]);
        let expected_selector = decode_selector(cols[2]);
        let expected_inner = decode_selector(cols[3]);
        assert_eq!(
            selector_prefixes_for_tests(&input),
            (expected_selector, expected_inner),
            "fixture mismatch for case_id={}",
            cols[0]
        );
        seen += 1;
    }

    assert!(seen > 0, "fixture must contain at least one case");
}
