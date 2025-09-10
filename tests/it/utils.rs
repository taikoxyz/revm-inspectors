use alloy_primitives::{Address, Bytes, B256};
use colorchoice::ColorChoice;
use revm::{
    context::{BlockEnv, CfgEnv, Evm, TxEnv, TxKind},
    context_interface::{
        result::{ExecutionResult, HaltReason},
    },
    database_interface::{MultiChainDatabase, MultiChainDatabaseCommit},
    handler::{instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr},
    interpreter::interpreter::EthInterpreter,
    primitives::{hardfork::SpecId, ChainAddress},
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
pub fn deploy_contract<DB: MultiChainDatabase + MultiChainDatabaseCommit>(
    evm: &mut EvmDb<DB, ()>,
    code: Bytes,
    deployer: Address,
    spec: SpecId,
) -> ExecutionResult<HaltReason> {
    evm.ctx().tx.caller = ChainAddress(1, deployer);
    evm.ctx().tx.gas_limit = 1000000;
    evm.ctx().tx.kind = TxKind::Create;
    evm.ctx().tx.data = code;
    evm.ctx().cfg.spec = spec;
    
    // Set prevrandao for post-Merge specs
    if spec >= SpecId::MERGE {
        use revm::context::BlockEnv;
        evm.ctx().block.entry(1).or_insert_with(BlockEnv::default).prevrandao = Some(B256::ZERO);
    }

    let out = evm.replay_commit().expect("Expect to be executed");
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
    
    evm.ctx().tx = TxEnv {
        caller: ChainAddress(1, deployer),
        gas_limit: 1000000,
        kind: TxKind::Create,
        data: code,
        ..Default::default()
    };
    let output = evm.inspect_replay_commit().expect("Expect to be executed");

    evm.ctx().tx.nonce += 1;
    output
}
