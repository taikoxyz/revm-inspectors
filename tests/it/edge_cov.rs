//! Edge coverage tests

use alloy_primitives::{hex, Address, B256, U256};
use revm::{
    context::{BlockEnv, TxEnv},
    context_interface::{
        result::{ExecutionResult, Output},
        ContextTr,
    },
    database::{CacheDB, SimpleMultiChainDB},
    database_interface::{EmptyDB, MultiChainDatabaseCommit},
    handler::EvmTr,
    inspector::InspectorEvmTr,
    primitives::{hardfork::SpecId, ChainAddress, MultiChainTxKind},
    Context, ExecuteEvm, InspectEvm, MainBuilder, MainContext,
};
use revm_inspectors::{
    edge_cov::EdgeCovInspector,
    tracing::{TracingInspector, TracingInspectorConfig},
};

#[test]
fn test_edge_coverage() {
    /*
    contract X {
        function Y(bool yes) external {
            for (uint256 i = 0; i < 255; i++) {
                if (yes) {
                    break;
                }
            }
        }
    }
    */

    let code = hex!("6080604052348015600f57600080fd5b5060b580601d6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063f42e8cdd14602d575b600080fd5b603c60383660046058565b603e565b005b60005b60ff811015605457816054576001016041565b5050565b600060208284031215606957600080fd5b81358015158114607857600080fd5b939250505056fea2646970667358221220a206d90c473b6930258d5789495c41b79941b5334c47a76b6e618d3571716d5164736f6c634300081c0033");
    let deployer = Address::ZERO;

    let mut multi_db = SimpleMultiChainDB::new();
    multi_db.add_chain(1, CacheDB::new(EmptyDB::default()));
    multi_db.add_chain(0, CacheDB::new(EmptyDB::default()));

    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| {
            cfg.chain_id = 1;
            cfg.spec = SpecId::LONDON;
        })
        .with_db(multi_db)
        .modify_block_chained(|blocks| {
            blocks.entry(1).or_insert_with(BlockEnv::default).prevrandao = Some(B256::ZERO);
        });

    let mut insp = TracingInspector::new(TracingInspectorConfig::default_geth());

    let mut evm = ctx.build_mainnet_with_inspector(&mut insp);

    // Create contract
    let res = evm
        .inspect_tx(TxEnv {
            caller: ChainAddress(1, deployer),
            gas_limit: 1000000,
            kind: MultiChainTxKind::Create,
            data: code.into(),
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
        })
        .unwrap();
    let addr = match res.result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Create(_, addr) => addr.unwrap(),
            _ => panic!("Create failed"),
        },
        _ => panic!("Execution failed"),
    };
    evm.ctx().db_mut().commit_multi(res.state);

    let acc = evm.ctx().db_mut().get_chain_mut(1).unwrap().load_account(deployer).unwrap();
    acc.info.balance = U256::from(u64::MAX);

    let tx = TxEnv {
        caller: ChainAddress(1, deployer),
        gas_limit: 100000000,
        kind: MultiChainTxKind::Call(ChainAddress(1, addr)),
        nonce: 1,
        // 'cast cd "Y(bool)" true'
        data: hex!("f42e8cdd0000000000000000000000000000000000000000000000000000000000000001")
            .into(),
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
    };

    let insp = EdgeCovInspector::new();
    let mut evm = evm.with_inspector(insp);
    evm.ctx().tx = tx;
    let tx = evm.ctx().tx.clone();
    let res = evm.inspect_tx(tx).unwrap();
    assert!(res.result.is_success());

    let counts = evm.inspector().get_hitcount();
    assert_eq!(counts.iter().filter(|&x| *x != 0).count(), 11);
    assert_eq!(counts.iter().filter(|&x| *x == 1).count(), 11);

    evm.inspector().reset();
    let res = evm
        .inspect_tx(TxEnv {
            caller: ChainAddress(1, deployer),
            gas_limit: 100000000,
            kind: MultiChainTxKind::Call(ChainAddress(1, addr)),
            nonce: 1,
            // 'cast cd "Y(bool)" false'
            data: hex!("f42e8cdd0000000000000000000000000000000000000000000000000000000000000000")
                .into(),
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
        })
        .unwrap();
    assert!(res.result.is_success());

    // There should be 13 non-zero counts and two edges that have been hit 255 times.
    let mut counts = evm.inspector.into_hitcount();

    counts.sort();
    assert_eq!(counts[counts.len() - 1], 255);
    assert_eq!(counts[counts.len() - 2], 255);
    assert_eq!(counts.iter().filter(|&x| *x != 0).count(), 13);
}
