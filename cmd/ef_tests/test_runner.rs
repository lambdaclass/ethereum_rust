use std::{collections::HashMap, path::Path};

use crate::types::TestUnit;
use ethereum_rust_core::{
    rlp::{decode::RLPDecode, encode::RLPEncode},
    types::{Account as CoreAccount, Block as CoreBlock},
};
use ethereum_rust_evm::{evm_state, execute_block, EvmState};
use ethereum_rust_storage::{EngineType, Store};

/// Tests the execute_block function
pub fn execute_test(test_key: &str, test: &TestUnit) {
    // Build pre state
    let mut evm_state = build_evm_state_for_test(test);
    let blocks = test.blocks.clone();

    // Check world_state
    check_prestate_against_db(test_key, test, evm_state.database());

    // Setup chain config
    let chain_config = test.network.chain_config();
    evm_state
        .database()
        .set_chain_config(chain_config)
        .expect("Failed to write to DB");

    // Execute all blocks in test
    for block_fixture in blocks.iter() {
        let block: &CoreBlock = &block_fixture.block().clone().into();

        let execution_result = execute_block(block, &mut evm_state);
        if block_fixture.expect_exception.is_some() {
            assert!(
                execution_result.is_err(),
                "Expected transaction execution to fail on test: {}",
                test_key
            )
        } else {
            assert!(
                execution_result.is_ok(),
                "Transaction execution failed on test: {} with error: {}",
                test_key,
                execution_result.unwrap_err()
            )
        }
    }
    check_poststate_against_db(test_key, test, evm_state.database())
}

pub fn parse_test_file(path: &Path) -> HashMap<String, TestUnit> {
    let s: String = std::fs::read_to_string(path).expect("Unable to read file");
    let tests: HashMap<String, TestUnit> = serde_json::from_str(&s).expect("Unable to parse JSON");
    tests
}

pub fn validate_test(test: &TestUnit) {
    // check that the decoded genesis block header matches the deserialized one
    let genesis_rlp = test.genesis_rlp.clone();
    let decoded_block = CoreBlock::decode(&genesis_rlp).unwrap();
    assert_eq!(
        decoded_block.header,
        test.genesis_block_header.clone().into()
    );

    // check that blocks can be decoded
    for block in &test.blocks {
        match CoreBlock::decode(block.rlp.as_ref()) {
            Ok(decoded_block) => {
                // check that the decoded block matches the deserialized one
                assert_eq!(decoded_block, (block.block().clone()).into());
                let mut rlp_block = Vec::new();
                // check that encoding the decoded block matches the rlp field
                decoded_block.encode(&mut rlp_block);
                assert_eq!(rlp_block, block.rlp.to_vec());
            }
            Err(_) => assert!(block.expect_exception.is_some()),
        }
    }
}

/// Creates an in-memory DB for evm execution and loads the prestate accounts
pub fn build_evm_state_for_test(test: &TestUnit) -> EvmState {
    let mut store =
        Store::new("store.db", EngineType::InMemory).expect("Failed to build DB for testing");
    let block_number = test.genesis_block_header.number.as_u64();
    store
        .add_block_header(block_number, test.genesis_block_header.clone().into())
        .unwrap();
    store
        .add_block_number(test.genesis_block_header.hash, block_number)
        .unwrap();
    let _ = store.update_latest_block_number(block_number);
    for (address, account) in &test.pre {
        let account: CoreAccount = account.clone().into();
        store
            .add_account(*address, account)
            .expect("Failed to write to test DB")
    }
    evm_state(store)
}

/// Checks db is correct after setting up initial state
/// Panics if any comparison fails
fn check_prestate_against_db(test_key: &str, test: &TestUnit, db: &Store) {
    let block_number = test.genesis_block_header.number.low_u64();
    let db_block_header = db.get_block_header(block_number).unwrap().unwrap();
    let test_state_root = test.genesis_block_header.state_root;
    assert_eq!(
        test_state_root, db_block_header.state_root,
        "Mismatched genesis state root for database, test: {test_key}"
    );
    assert_eq!(
        test_state_root,
        db.clone().world_state_root(),
        "Mismatched genesis state root for world state trie, test: {test_key}"
    );
}

/// Checks that all accounts in the post-state are present and have the correct values in the DB
/// Panics if any comparison fails
fn check_poststate_against_db(test_key: &str, test: &TestUnit, db: &Store) {
    for (addr, account) in &test.post_state {
        let expected_account: CoreAccount = account.clone().into();
        // Check info
        let db_account_info = db
            .get_account_info(*addr)
            .expect("Failed to read from DB")
            .unwrap_or_else(|| {
                panic!("Account info for address {addr} not found in DB, test:{test_key}")
            });
        assert_eq!(
            db_account_info, expected_account.info,
            "Mismatched account info for address {addr} test:{test_key}"
        );
        // Check code
        let code_hash = expected_account.info.code_hash;
        let db_account_code = db
            .get_account_code(code_hash)
            .expect("Failed to read from DB")
            .unwrap_or_else(|| {
                panic!("Account code for code hash {code_hash} not found in DB test:{test_key}")
            });
        assert_eq!(
            db_account_code, expected_account.code,
            "Mismatched account code for code hash {code_hash} test:{test_key}"
        );
        // Check storage
        for (key, value) in expected_account.storage {
            let db_storage_value = db
                .get_storage_at(*addr, key)
                .expect("Failed to read from DB")
                .unwrap_or_else(|| {
                    panic!("Storage missing for address {addr} key {key} in DB test:{test_key}")
                });
            assert_eq!(
                db_storage_value, value,
                "Mismatched storage value for address {addr}, key {key} test:{test_key}"
            );
        }
    }
    // Check lastblockhash is in store
    let last_block_number = db.get_latest_block_number().unwrap().unwrap();
    let last_block_hash = db
        .get_block_header(last_block_number)
        .unwrap()
        .unwrap()
        .compute_block_hash();
    assert_eq!(
        test.lastblockhash, last_block_hash,
        "Last block number does not match"
    );
    // Get block header
    let last_block = db.get_block_header(last_block_number).unwrap();
    assert!(last_block.is_some(), "Block hash is not stored in db");
    // Check world state
    let db_state_root = last_block.unwrap().state_root;
    assert_eq!(
        db_state_root,
        db.clone().world_state_root(),
        "Mismatched state root for world state trie, test: {test_key}"
    );
}
