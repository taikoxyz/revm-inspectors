use alloy_primitives::{Address, Bytes, B256, U256};
use colorchoice::ColorChoice;
use revm::{
    context::{BlockEnv, CfgEnv, Evm, TxEnv},
    context_interface::result::{ExecutionResult, HaltReason},
    database_interface::{Database, DatabaseCommit},
    handler::{instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr},
    interpreter::interpreter::EthInterpreter,
    primitives::{hardfork::SpecId, TxKind},
    Context, ExecuteCommitEvm, InspectCommitEvm, Inspector, Journal,
};

use revm_inspectors::tracing::{TraceWriter, TraceWriterConfig, TracingInspector};

pub type ContextDb<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB, Journal<DB>, ()>;

pub fn write_traces(tracer: &TracingInspector) -> String {
    write_traces_with(tracer, TraceWriterConfig::new().color_choice(ColorChoice::Never))
}

pub fn write_traces_with(tracer: &TracingInspector, config: TraceWriterConfig) -> String {
    let mut w = TraceWriter::with_config(Vec::<u8>::new(), config);
    w.write_arena(tracer.traces()).expect("failed to write traces to Vec<u8>");
    String::from_utf8(w.into_writer()).expect("trace writer wrote invalid UTF-8")
}

pub fn print_traces(tracer: &TracingInspector) {
    // Use `println!` so that the output is captured by the test runner.
    println!("{}", write_traces_with(tracer, TraceWriterConfig::new()));
}

pub type EvmDb<DB, INSP> = Evm<
    ContextDb<DB>,
    INSP,
    EthInstructions<EthInterpreter, ContextDb<DB>>,
    EthPrecompiles,
    EthFrame,
>;

/// Deploys a contract with the given code and deployer address.
/// This function assumes the deployer account already has proper balance and nonce tracking
pub fn deploy_contract<DB: Database + DatabaseCommit>(
    evm: &mut EvmDb<DB, ()>,
    code: Bytes,
    deployer: Address,
    spec: SpecId,
) -> ExecutionResult<HaltReason> {
    // Try to get current nonce, but we can't access it generically
    // The caller should ensure proper nonce is set before calling
    // For first deployment, caller should use nonce 0
    // For subsequent deployments, nonce should be incremented
    let current_nonce = if evm.ctx().tx.caller == deployer {
        // Same deployer, increment nonce
        evm.ctx().tx.nonce
    } else {
        // Different deployer or first deployment, start from 0
        0
    };

    evm.ctx().tx.caller = deployer;
    evm.ctx().tx.gas_limit = 1000000;
    evm.ctx().tx.kind = TxKind::Create;
    evm.ctx().tx.data = code;
    evm.ctx().tx.nonce = current_nonce;
    evm.ctx().cfg.spec = spec;

    // Set prevrandao for post-Merge specs
    if spec >= SpecId::MERGE {
        evm.ctx().block.prevrandao = Some(B256::ZERO);
    }

    let tx = evm.ctx().tx.clone();
    let out = evm.transact_commit(tx).expect("Expect to be executed");
    // Increment nonce for next deployment with same deployer
    evm.ctx().tx.nonce += 1;
    out
}

/// Deploys a contract with the given code and deployer address.
pub fn inspect_deploy_contract<
    DB: Database + DatabaseCommit,
    INSP: Inspector<ContextDb<DB>>,
>(
    evm: &mut EvmDb<DB, INSP>,
    code: Bytes,
    deployer: Address,
    spec: SpecId,
) -> ExecutionResult<HaltReason> {
    evm.ctx().cfg.spec = spec;

    // Set prevrandao for post-Merge specs
    if spec >= SpecId::MERGE {
        evm.ctx().block.prevrandao = Some(B256::ZERO);
    }

    // Same logic as deploy_contract - track nonce based on deployer
    let current_nonce = if evm.ctx().tx.caller == deployer { evm.ctx().tx.nonce } else { 0 };

    let output = evm
        .inspect_tx_commit(TxEnv {
            caller: deployer,
            gas_limit: 1000000,
            kind: TxKind::Create,
            data: code,
            nonce: current_nonce,
            value: U256::ZERO,
            gas_price: 0,
            chain_id: Some(1),
            tx_type: 0,
            access_list: Default::default(),
            gas_priority_fee: None,
            max_fee_per_blob_gas: 0,
            blob_hashes: Default::default(),
            authorization_list: Default::default(),
        })
        .expect("Expect to be executed");

    evm.ctx().tx.nonce += 1;
    output
}
