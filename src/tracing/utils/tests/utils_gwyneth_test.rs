use super::*;

#[test]
fn decode_revert_reason_with_null_bytes() {
    let empty_err = GenericContractError::Revert("".into());
    let encoded = empty_err.abi_encode();
    assert!(maybe_revert_reason(&encoded).is_none());

    let null_bytes_err =
        GenericContractError::Revert(String::from_utf8(vec![0u8; 32]).unwrap().into());
    let encoded = null_bytes_err.abi_encode();
    assert!(maybe_revert_reason(&encoded).is_none());
}

#[test]
fn decode_revert_reason_with_invalid_selector() {
    // Test data that doesn't start with revert or panic selector
    let invalid_data = vec![0x12, 0x34, 0x56, 0x78]; // Invalid selector
    assert_eq!(
        maybe_revert_reason(&invalid_data),
        None,
        "Should return None for invalid selector"
    );

    // Test data too short
    let short_data = vec![0x08, 0xc3]; // Only 2 bytes
    assert_eq!(maybe_revert_reason(&short_data), None, "Should return None for data too short");

    // Test valid string but with proper selector validation
    let control_char_err = GenericContractError::Revert("\u{000f}.[l".into());
    let encoded = control_char_err.abi_encode();
    // This should now return the string since we're using Geth's validation logic
    assert_eq!(maybe_revert_reason(&encoded), Some("\u{000f}.[l".to_string()));

    // Test normal readable string
    let readable_err = GenericContractError::Revert("Normal error message".into());
    let encoded = readable_err.abi_encode();
    assert_eq!(maybe_revert_reason(&encoded), Some("Normal error message".to_string()));
}

#[test]
fn decode_revert_reason_with_raw_string() {
    let non_readable_data = "\u{9a62}\u{0002}".as_bytes();
    assert_eq!(
        maybe_revert_reason(non_readable_data),
        None,
        "Should return None for raw strings"
    );
}

