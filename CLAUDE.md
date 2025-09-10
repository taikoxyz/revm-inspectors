# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo check` - Quick compile check without generating executable

### Testing
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test containing the name
- `cargo test --no-run` - Compile tests but don't run them

### Code Quality
- `cargo clippy` - Run Clippy linter with project-specific rules (see clippy.toml)
- `cargo clippy --fix` - Apply Clippy suggestions automatically
- `cargo fmt` - Format code according to rustfmt.toml configuration
- `cargo fmt --check` - Check if code is properly formatted

### Features
- `cargo build --features serde` - Build with serde serialization support
- `cargo build --features js-tracer` - Build with JavaScript tracer support (requires boa_engine)

## Architecture Overview

This crate provides Inspector implementations for the REVM (Rust EVM) to trace and monitor EVM execution. It was originally part of Reth as `reth-revm-inspectors`.

### Core Components

1. **TracingInspector** (`src/tracing/mod.rs:56-734`) - Main tracing inspector that implements the revm::Inspector trait
   - Tracks call execution, steps, logs, and state changes
   - Configurable via TracingInspectorConfig for different tracing needs
   - Builds traces that can be converted to Geth or Parity formats

2. **Trace Builders** (`src/tracing/builder/`)
   - **GethTraceBuilder** - Converts traces to Geth-compatible format
   - **ParityTraceBuilder** - Converts traces to Parity-compatible format

3. **Specialized Inspectors**
   - **AccessListInspector** (`src/access_list.rs`) - EIP-2930 access list generation
   - **TransferInspector** (`src/transfer.rs`) - Tracks internal value transfers
   - **OpcodeCountInspector** (`src/tracing/opcount.rs`) - Counts opcode execution
   - **FourByteInspector** (`src/tracing/fourbyte.rs`) - Tracks function selectors

4. **JavaScript Tracer** (`src/tracing/js/` - feature gated)
   - Supports custom JavaScript-based tracing logic
   - Uses boa_engine for JavaScript execution

5. **MuxInspector** (`src/tracing/mux.rs`) - Multiplexes multiple inspectors

### Key Dependencies
- `revm` - The core EVM implementation (using Taiko fork)
- `alloy-*` - Ethereum types and RPC interfaces
- `boa_engine` - JavaScript engine (optional, js-tracer feature)

### Chain Configuration
The crate uses a global CHAIN_ID static that defaults to 1 (mainnet). Call `set_chain_id()` before using inspectors to configure for other chains.

### Configuration Files
- `clippy.toml` - Clippy linting configuration with MSRV 1.79
- `rustfmt.toml` - Code formatting rules (100 char width, import granularity)
- `deny.toml` - Dependency auditing configuration