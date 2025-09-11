
use alloy_primitives::{Address, Bytes, B256, U256};
use colorchoice::ColorChoice;
use revm::{
    context::{BlockEnv, CfgEnv, Evm, TxEnv},
    context_interface::{
        result::{ExecutionResult, HaltReason},
    },
    database_interface::{MultiChainDatabase, MultiChainDatabaseCommit},
    handler::{instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr},
    interpreter::interpreter::EthInterpreter,
    primitives::{hardfork::SpecId, ChainAddress, MultiChainTxKind},
    Context, ExecuteCommitEvm, InspectCommitEvm, InspectEvm, Inspector, Journal,
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
pub fn deploy_contract<DB: MultiChainDatabase + MultiChainDatabaseCommit>(
    evm: &mut EvmDb<DB, ()>,
    code: Bytes,
    deployer: Address,
    spec: SpecId,
) -> ExecutionResult<HaltReason> {
    evm.ctx().tx.caller = ChainAddress(1, deployer);
    evm.ctx().tx.gas_limit = 1000000;
    evm.ctx().tx.kind = MultiChainTxKind::Create;
    evm.ctx().tx.data = code;
    evm.ctx().tx.nonce = 0;
    evm.ctx().cfg.spec = spec;
    
    // Set prevrandao for post-Merge specs
    if spec >= SpecId::MERGE {
        use revm::context::BlockEnv;
        evm.ctx().block.entry(1).or_insert_with(BlockEnv::default).prevrandao = Some(B256::ZERO);
    }

    let tx = evm.ctx().tx.clone();
    let out = evm.transact_commit(tx).expect("Expect to be executed");
    evm.ctx().tx.nonce += 1;
    out
}

/// Deploys a contract with the given code and deployer address.
pub fn inspect_deploy_contract<DB: MultiChainDatabase + MultiChainDatabaseCommit, INSP: Inspector<ContextDb<DB>>>(
    evm: &mut EvmDb<DB, INSP>,
    code: Bytes,
    deployer: Address,
    spec: SpecId,
) -> ExecutionResult<HaltReason> {
    evm.ctx().cfg.spec = spec;
    
    // Set prevrandao for post-Merge specs
    if spec >= SpecId::MERGE {
        use revm::context::BlockEnv;
        evm.ctx().block.entry(1).or_insert_with(BlockEnv::default).prevrandao = Some(B256::ZERO);
    }
    
    let output = evm.inspect_tx_commit(TxEnv {
        caller: ChainAddress(1, deployer),
        gas_limit: 1000000,
        kind: MultiChainTxKind::Create,
        data: code,
        nonce: 0,
        value: U256::ZERO,
        gas_price: 0,
        chain_id: Some(1),
        chain_ids: Some(vec![1]),
        tx_type: 0,
        access_list: Default::default(),
        gas_priority_fee: None,
        max_fee_per_blob_gas: 0,
        blob_hashes: Default::default(),
        authorization_list: Default::default(),
    }).expect("Expect to be executed");

    evm.ctx().tx.nonce += 1;
    output
}
