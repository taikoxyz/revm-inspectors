//! Accesslist tests

use alloy_primitives::{address, hex, B256, U256};
use revm::{
    bytecode::Bytecode,
    context::{BlockEnv, TxEnv},
    database::{CacheDB, SimpleMultiChainDB},
    database_interface::EmptyDB,
    handler::EvmTr,
    primitives::{ChainAddress, MultiChainTxKind},
    state::AccountInfo,
    Context, InspectEvm, MainBuilder, MainContext,
};
use revm_inspectors::access_list::AccessListInspector;

#[test]
fn test_access_list_precompile() {
    /*
    contract Storage {
       function recoverSignature() public view returns (address) {
            address r = ecrecover(bytes32(0), 0, 0);
        }
    }
    */

    let code = hex!("608060405234801561000f575f80fd5b5060dd80601a5f395ff3fe608060405260043610601b575f3560e01c8063a53997051461001f575b5f80fd5b602e602a3660046074565b6030565b005b6040518181526001600160a01b0383169033907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9060200160405180910390a35050565b5f80604083850312156084575f80fd5b82356001600160a01b03811681146099575f80fd5b94602093909301359350505056fea2646970667358221220d81408f997c5f148e7d6afc66ccc7cda17a38396925363f11993fa885b70729b64736f6c63430008190033");

    let account = address!("341348115259a8bf69f1f50101c227fced83bac6");
    let caller = address!("341348115259a8bf69f1f50101c227fced83bac5");

    let mut multi_db = SimpleMultiChainDB::new();
    multi_db.add_chain(1, CacheDB::new(EmptyDB::default()));
    multi_db.get_chain_mut(1).unwrap().insert_account_info(
        account,
        AccountInfo { code: Some(Bytecode::new_raw(code.into())), ..Default::default() },
    );

    let context = Context::mainnet()
        .with_db(multi_db)
        .modify_block_chained(|blocks| {
            blocks.entry(1).or_insert_with(BlockEnv::default).prevrandao = Some(B256::ZERO);
        });
    let mut evm = context.build_mainnet();
    
    evm.ctx().tx = TxEnv {
        caller: ChainAddress(1, caller),
        gas_limit: 1000000,
        kind: MultiChainTxKind::Call(ChainAddress(1, account)),
        data: hex!("a5399705").into(),
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
    };
    let mut accesslist = AccessListInspector::default();
    let mut evm = evm.with_inspector(&mut accesslist);
    let tx = evm.ctx().tx.clone();
    let res = evm.inspect_tx(tx).unwrap();
    assert!(res.result.is_success(), "{res:#?}");

    let erecover = address!("0x0000000000000000000000000000000000000001");
    assert!(accesslist.excluded().contains(&erecover));
    assert!(accesslist.into_access_list().is_empty());
}