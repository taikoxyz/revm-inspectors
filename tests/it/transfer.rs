//! Transfer tests

use alloy_primitives::{hex, Address, U256};
use revm::{
    context::TxEnv,
    context::TxKind,
    database::MultiCacheDB,
    database_interface::EmptyDB,
    primitives::{hardfork::SpecId, ChainAddress},
    state::AccountInfo,
    Context, InspectCommitEvm, MainBuilder, MainContext,
};
use revm_inspectors::transfer::{TransferInspector, TransferKind, TransferOperation};

use crate::utils::deploy_contract;

#[test]
fn test_internal_transfers() {
    /*
    contract Transfer {

        function sendViaCall(address payable _to) public payable {
            (bool sent, bytes memory data) = _to.call{value: msg.value}("");
        }
    }
    */

    let code = hex!("608060405234801561001057600080fd5b5060ef8061001f6000396000f3fe608060405260043610601c5760003560e01c8063830c29ae146021575b600080fd5b6030602c366004608b565b6032565b005b600080826001600160a01b03163460405160006040518083038185875af1925050503d8060008114607e576040519150601f19603f3d011682016040523d82523d6000602084013e6083565b606091505b505050505050565b600060208284031215609c57600080fd5b81356001600160a01b038116811460b257600080fd5b939250505056fea26469706673582212201654bdbf09c088897c9b02f3ba9df280b136ef99c3a05ca5a21d9a10fd912d3364736f6c634300080d0033");
    let deployer = Address::ZERO;

    let mut multi_db = MultiCacheDB::new();
    multi_db.add_chain(1, EmptyDB::default());
    
    // Insert deployer account with balance first
    multi_db.get_chain_mut(1).unwrap().insert_account_info(deployer, AccountInfo { balance: U256::from(u64::MAX), ..Default::default() });

    let mut evm = Context::mainnet()
        .with_db(multi_db)
        .build_mainnet();

    // Deploy contract using utility function
    let res = deploy_contract(&mut evm, code.into(), deployer, SpecId::LONDON);
    assert!(res.is_success(), "Contract deployment failed: {:?}", res);
    let addr = res.created_address().unwrap();
    println!("Deployed contract to address: {:?}", addr);

    // First test: with all transfers
    let mut transfer_inspector = TransferInspector::new(false);
    evm.ctx.tx = TxEnv {
        caller: ChainAddress(1, deployer),
        gas_limit: 100000000,
        kind: TxKind::Call(ChainAddress(1, addr)),
        data: hex!("830c29ae0000000000000000000000000000000000000000000000000000000000000000")
            .into(),
        value: U256::from(10),
        nonce: 1,
        ..Default::default()
    };
    evm.ctx.cfg.spec = SpecId::LONDON;
    let res = evm.with_inspector(&mut transfer_inspector).inspect_replay_commit().unwrap();
    assert!(res.is_success());

    assert_eq!(transfer_inspector.transfers().len(), 2);
    assert_eq!(
        transfer_inspector.transfers()[0],
        TransferOperation {
            kind: TransferKind::Call,
            from: ChainAddress(1, deployer),
            to: ChainAddress(1, addr),
            value: U256::from(10),
        }
    );
    assert_eq!(
        transfer_inspector.transfers()[1],
        TransferOperation {
            kind: TransferKind::Call,
            from: ChainAddress(1, addr),
            to: ChainAddress(1, deployer),
            value: U256::from(10),
        }
    );

    // Second test: with internal transfers only
    // Since evm was consumed, we need to create a new one with the database
    // that has the deployed contract
    let mut internal_inspector = TransferInspector::internal_only();
    
    // Get the database from the original evm's journal
    // Actually, we can't access it after with_inspector consumed it
    // So let's just run the second test without creating a new evm
    // We'll use a different contract call to test internal_only
    
    // Deploy and call the contract again for the internal_only test
    let mut multi_db2 = MultiCacheDB::new();
    multi_db2.add_chain(1, EmptyDB::default());
    multi_db2.get_chain_mut(1).unwrap().insert_account_info(deployer, AccountInfo { balance: U256::from(u64::MAX), ..Default::default() });
    
    let mut evm2 = Context::mainnet()
        .with_db(multi_db2)
        .build_mainnet();
    
    // Deploy contract again
    let res2 = deploy_contract(&mut evm2, code.into(), deployer, SpecId::LONDON);
    assert!(res2.is_success(), "Second contract deployment failed: {:?}", res2);
    let addr2 = res2.created_address().unwrap();
    println!("Deployed second contract to address: {:?}", addr2);
    
    // Set up transaction for internal_only test
    evm2.ctx.tx = TxEnv {
        caller: ChainAddress(1, deployer),
        gas_limit: 100000000,
        kind: TxKind::Call(ChainAddress(1, addr2)),
        data: hex!("830c29ae0000000000000000000000000000000000000000000000000000000000000000")
            .into(),
        value: U256::from(10),
        nonce: 1,
        ..Default::default()
    };
    evm2.ctx.cfg.spec = SpecId::LONDON;
    let res = evm2.with_inspector(&mut internal_inspector).inspect_replay_commit().unwrap();
    assert!(res.is_success());

    assert_eq!(internal_inspector.transfers().len(), 1);
    assert_eq!(
        internal_inspector.transfers()[0],
        TransferOperation {
            kind: TransferKind::Call,
            from: ChainAddress(1, addr2),
            to: ChainAddress(1, deployer),
            value: U256::from(10),
        }
    );
}
