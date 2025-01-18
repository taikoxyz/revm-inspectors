//! revm [Inspector](revm::Inspector) implementations, such as call tracers
//!
//! ## Feature Flags
//!
//! - `js-tracer`: Enables a JavaScript tracer implementation. This pulls in extra dependencies
//!   (such as `boa`, `tokio` and `serde_json`).

#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Default to chain id 1 (mainnet)
/// Reth should call `set_chain_id` to set the chain id before running revm-inspectors.
pub static CHAIN_ID: LazyLock<Mutex<u64>> = std::sync::LazyLock::new(|| Mutex::new(1));

/// An inspector implementation for an EIP2930 Accesslist
pub mod access_list;

/// implementation of an opcode counter for the EVM.
pub mod opcode;

/// An inspector for recording traces
pub mod tracing;

/// An inspector for recording internal transfers.
pub mod transfer;

use std::sync::{LazyLock, Mutex};

use alloy_primitives::Address;
pub use colorchoice::ColorChoice;
use revm::primitives::ChainAddress;

pub fn set_chain_id(id: u64) {
    *CHAIN_ID.lock().unwrap() = id;
}

pub fn get_chain_id() -> u64 {
    *CHAIN_ID.lock().unwrap()
}

pub fn chain_address(addr: Address) -> ChainAddress {
    ChainAddress(get_chain_id(), addr)
}
