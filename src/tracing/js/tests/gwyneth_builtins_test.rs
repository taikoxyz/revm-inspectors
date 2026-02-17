use super::*;

use alloy_primitives::bytes;
use serde_json::json;

#[test]
fn test_slice_builtin() {
    let code = r#"{
        res: [],
        step: function(log) {
            // Test slicing a hex string
            var hex = '0xdeadbeefcafe';
            this.res.push(toHex(slice(hex, 0, 2)));
            this.res.push(toHex(slice(hex, 2, 4)));
            this.res.push(toHex(slice(hex, 4, 6)));

            // Test slicing an array
            var arr = [0x01, 0x02, 0x03, 0x04, 0x05];
            this.res.push(toHex(slice(arr, 0, 3)));
            this.res.push(toHex(slice(arr, 1, 4)));

            // Test slicing a Uint8Array
            var uint8 = new Uint8Array([0xff, 0xee, 0xdd, 0xcc, 0xbb]);
            this.res.push(toHex(slice(uint8, 0, 2)));
            this.res.push(toHex(slice(uint8, 2, 5)));
        },
        fault: function() {},
        result: function() { return this.res }
    }"#;
    let res = run_trace(code, Some(bytes!("0x00")), true);
    assert_eq!(
        res,
        json!(["0xdead", "0xbeef", "0xcafe", "0x010203", "0x020304", "0xffee", "0xddccbb"])
    );
}

#[test]
fn test_is_precompiled_builtin() {
    let code = r#"{
        res: [],
        step: function(log) {
            this.res.push(isPrecompiled("0x01"));
            this.res.push(isPrecompiled("0x0000000000000000000000000000000000000002"));
            this.res.push(isPrecompiled("0x0000000000000000000000000000000000000000"));
        },
        fault: function() {},
        result: function() { return this.res }
    }"#;
    let res = run_trace(code, Some(bytes!("0x00")), true);
    assert_eq!(res, json!([true, true, false]));
}

#[test]
fn test_has_own_property() {
    let code = r#"{
        res: [],
        step: function(log) {
            this.res.push(log.hasOwnProperty("stack"));
        },
        fault: function() {},
        result: function() { return this.res }
    }"#;
    let res = run_trace(code, Some(bytes!("0x00")), true);
    assert_eq!(res, json!([true]));
}

#[test]
fn test_slice_with_stack_values() {
    let code = r#"{
        res: [],
        step: function(log) {
            if ((log.stack.length() > 0) && log.memory.length() >= log.stack.peek(0)) {
                this.res.push(log.memory.slice(0, log.stack.peek(0)));
            }
        },
        fault: function() {},
        result: function() { return this.res }
    }"#;
    let res = run_trace(code, Some(bytes!("0x5F5F52600100")), true);
    assert_eq!(res, json!([json!({}), json!({}), json!({"0": 0})]));
}

