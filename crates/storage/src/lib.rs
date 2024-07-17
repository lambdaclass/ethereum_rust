//! Abstraction for persistent data storage.
//!    Supporting InMemory and Libmdbx storage.
//!    There is also a template for Sled and RocksDb implementation in case we
//!    want to test or benchmark against those engines (Currently disabled behind feature flags
//!    to avoid requiring the implementation of the full API).

use self::error::StoreError;
#[cfg(feature = "in_memory")]
use self::in_memory::Store as InMemoryStore;
#[cfg(feature = "libmdbx")]
use self::libmdbx::Store as LibmdbxStore;
#[cfg(feature = "rocksdb")]
use self::rocksdb::Store as RocksDbStore;
#[cfg(feature = "sled")]
use self::sled::Store as SledStore;
use bytes::Bytes;
use ethereum_rust_core::types::{AccountInfo, BlockBody, BlockHash, BlockHeader, BlockNumber};
use ethereum_types::{Address, H256};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

mod error;
mod rlp;

#[cfg(feature = "in_memory")]
mod in_memory;
#[cfg(feature = "libmdbx")]
mod libmdbx;
#[cfg(feature = "rocksdb")]
mod rocksdb;
#[cfg(feature = "sled")]
mod sled;

pub(crate) type Key = Vec<u8>;
pub(crate) type Value = Vec<u8>;

pub trait StoreEngine: Debug + Send {
    /// Add account info
    fn add_account_info(
        &mut self,
        address: Address,
        account_info: AccountInfo,
    ) -> Result<(), StoreError>;

    /// Obtain account info
    fn get_account_info(&self, address: Address) -> Result<Option<AccountInfo>, StoreError>;

    /// Add block header
    fn add_block_header(
        &mut self,
        block_number: BlockNumber,
        block_header: BlockHeader,
    ) -> Result<(), StoreError>;

    /// Obtain block header
    fn get_block_header(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<BlockHeader>, StoreError>;

    /// Add block body
    fn add_block_body(
        &mut self,
        block_number: BlockNumber,
        block_body: BlockBody,
    ) -> Result<(), StoreError>;

    /// Obtain block body
    fn get_block_body(&self, block_number: BlockNumber) -> Result<Option<BlockBody>, StoreError>;

    /// Add block body
    fn add_block_number(
        &mut self,
        block_hash: BlockHash,
        block_number: BlockNumber,
    ) -> Result<(), StoreError>;

    /// Obtain block number
    fn get_block_number(&self, block_hash: BlockHash) -> Result<Option<BlockNumber>, StoreError>;

    /// Set an arbitrary value (used for eventual persistent values: eg. current_block_height)
    fn set_value(&mut self, key: Key, value: Value) -> Result<(), StoreError>;

    /// Retrieve a stored value under Key
    fn get_value(&self, key: Key) -> Result<Option<Value>, StoreError>;

    /// Add account code
    fn add_account_code(&mut self, code_hash: H256, code: Bytes) -> Result<(), StoreError>;

    /// Obtain account code via code hash
    fn get_account_code(&self, code_hash: H256) -> Result<Option<Bytes>, StoreError>;

    /// Obtain account code via account address
    fn get_code_by_account_address(&self, address: Address) -> Result<Option<Bytes>, StoreError> {
        let code_hash = match self.get_account_info(address) {
            Ok(Some(acc_info)) => acc_info.code_hash,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };
        self.get_account_code(code_hash)
    }

    // Add storage value
    fn add_storage_at(
        &mut self,
        address: Address,
        storage_key: H256,
        storage_value: H256,
    ) -> Result<(), StoreError>;

    // Obtain storage value
    fn get_storage_at(
        &self,
        address: Address,
        storage_key: H256,
    ) -> Result<Option<H256>, StoreError>;
}

#[derive(Debug, Clone)]
pub struct Store {
    engine: Arc<Mutex<dyn StoreEngine>>,
}

#[allow(dead_code)]
pub enum EngineType {
    #[cfg(feature = "in_memory")]
    InMemory,
    #[cfg(feature = "libmdbx")]
    Libmdbx,
    #[cfg(feature = "sled")]
    Sled,
    #[cfg(feature = "rocksdb")]
    RocksDb,
}

impl Store {
    pub fn new(path: &str, engine_type: EngineType) -> Result<Self, StoreError> {
        let store = match engine_type {
            #[cfg(feature = "libmdbx")]
            EngineType::Libmdbx => Self {
                engine: Arc::new(Mutex::new(LibmdbxStore::new(path)?)),
            },
            #[cfg(feature = "in_memory")]
            EngineType::InMemory => Self {
                engine: Arc::new(Mutex::new(InMemoryStore::new()?)),
            },
            #[cfg(feature = "sled")]
            EngineType::Sled => Self {
                engine: Arc::new(Mutex::new(SledStore::new(path)?)),
            },
            #[cfg(feature = "rocksdb")]
            EngineType::RocksDb => Self {
                engine: Arc::new(Mutex::new(RocksDbStore::new(path)?)),
            },
        };
        Ok(store)
    }

    pub fn add_account_info(
        &mut self,
        address: Address,
        account_info: AccountInfo,
    ) -> Result<(), StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_account_info(address, account_info)
    }

    pub fn get_account_info(&self, address: Address) -> Result<Option<AccountInfo>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_account_info(address)
    }

    pub fn add_block_header(
        &self,
        block_number: BlockNumber,
        block_header: BlockHeader,
    ) -> Result<(), StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_block_header(block_number, block_header)
    }

    pub fn get_block_header(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<BlockHeader>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block_header(block_number)
    }

    pub fn add_block_body(
        &self,
        block_number: BlockNumber,
        block_body: BlockBody,
    ) -> Result<(), StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_block_body(block_number, block_body)
    }

    pub fn get_block_body(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<BlockBody>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block_body(block_number)
    }

    pub fn add_block_number(
        &self,
        block_hash: BlockHash,
        block_number: BlockNumber,
    ) -> Result<(), StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_block_number(block_hash, block_number)
    }

    pub fn get_block_number(
        &self,
        block_hash: BlockHash,
    ) -> Result<Option<BlockNumber>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_block_number(block_hash)
    }

    pub fn add_account_code(&self, code_hash: H256, code: Bytes) -> Result<(), StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .add_account_code(code_hash, code)
    }

    pub fn get_account_code(&self, code_hash: H256) -> Result<Option<Bytes>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_account_code(code_hash)
    }

    pub fn get_code_by_account_address(
        &self,
        address: Address,
    ) -> Result<Option<Bytes>, StoreError> {
        self.engine
            .clone()
            .lock()
            .unwrap()
            .get_code_by_account_address(address)
    }

    pub fn add_storage_at(
        &self,
        address: Address,
        storage_key: H256,
        storage_value: H256,
    ) -> Result<(), StoreError>{
        self.engine.lock().unwrap().add_storage_at(address, storage_key, storage_value)
    }

    pub fn get_storage_at(
        &self,
        address: Address,
        storage_key: H256,
    ) -> Result<Option<H256>, StoreError>{
        self.engine.lock().unwrap().get_storage_at(address, storage_key)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, str::FromStr};

    use bytes::Bytes;
    use ethereum_rust_core::{
        rlp::decode::RLPDecode,
        types::{self, Transaction},
        Bloom,
    };
    use ethereum_types::{H256, U256};

    use super::*;

    #[cfg(feature = "in_memory")]
    #[test]
    fn test_in_memory_store() {
        let store = Store::new("test", EngineType::InMemory).unwrap();
        test_store_suite(store);
    }

    #[cfg(feature = "libmdbx")]
    #[test]
    fn test_libmdbx_store() {
        // Removing preexistent DBs in case of a failed previous test
        remove_test_dbs("test.mdbx");
        let store = Store::new("test.mdbx", EngineType::Libmdbx).unwrap();
        test_store_suite(store);
        remove_test_dbs("test.mdbx");
    }

    #[cfg(feature = "sled")]
    #[test]
    fn test_sled_store() {
        // Removing preexistent DBs in case of a failed previous test
        remove_test_dbs("test.sled");
        let store = Store::new("test.sled", EngineType::Sled).unwrap();
        test_store_suite(store);
        remove_test_dbs("test.sled");
    }

    #[cfg(feature = "rocksdb")]
    #[test]
    fn test_rocksdb_store() {
        // Removing preexistent DBs in case of a failed previous test
        remove_test_dbs("test.rocksdb");
        let store = Store::new("test.rocksdb", EngineType::Sled).unwrap();
        test_store_suite(store);
        remove_test_dbs("test.rocksdb");
    }

    fn test_store_suite(store: Store) {
        test_store_account(store.clone());
        test_store_block(store.clone());
        test_store_block_number(store.clone());
        test_store_account_code(store.clone());
        test_store_account_storage(store.clone());
    }

    fn test_store_account(mut store: Store) {
        let address = Address::random();
        let code = Bytes::new();
        let balance = U256::from_dec_str("50").unwrap();
        let nonce = 5;
        let code_hash = types::code_hash(&code);

        let account_info = new_account_info(code.clone(), balance, nonce);
        let _ = store.add_account_info(address, account_info);

        let stored_account_info = store.get_account_info(address).unwrap().unwrap();

        assert_eq!(code_hash, stored_account_info.code_hash);
        assert_eq!(balance, stored_account_info.balance);
        assert_eq!(nonce, stored_account_info.nonce);
    }

    fn new_account_info(code: Bytes, balance: U256, nonce: u64) -> AccountInfo {
        AccountInfo {
            code_hash: types::code_hash(&code),
            balance,
            nonce,
        }
    }

    fn remove_test_dbs(prefix: &str) {
        // Removes all test databases from filesystem
        for entry in fs::read_dir(env::current_dir().unwrap()).unwrap() {
            if entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with(prefix)
            {
                fs::remove_dir_all(entry.unwrap().path()).unwrap();
            }
        }
    }

    fn test_store_block(store: Store) {
        let (block_header, block_body) = create_block_for_testing();
        let block_number = 6;

        store
            .add_block_header(block_number, block_header.clone())
            .unwrap();
        store
            .add_block_body(block_number, block_body.clone())
            .unwrap();

        let stored_header = store.get_block_header(block_number).unwrap().unwrap();
        let stored_body = store.get_block_body(block_number).unwrap().unwrap();

        assert_eq!(stored_header, block_header);
        assert_eq!(stored_body, block_body);
    }

    fn create_block_for_testing() -> (BlockHeader, BlockBody) {
        let block_header = BlockHeader {
            parent_hash: H256::from_str(
                "0x1ac1bf1eef97dc6b03daba5af3b89881b7ae4bc1600dc434f450a9ec34d44999",
            )
            .unwrap(),
            ommers_hash: H256::from_str(
                "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            )
            .unwrap(),
            coinbase: Address::from_str("0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").unwrap(),
            state_root: H256::from_str(
                "0x9de6f95cb4ff4ef22a73705d6ba38c4b927c7bca9887ef5d24a734bb863218d9",
            )
            .unwrap(),
            transactions_root: H256::from_str(
                "0x578602b2b7e3a3291c3eefca3a08bc13c0d194f9845a39b6f3bcf843d9fed79d",
            )
            .unwrap(),
            receipt_root: H256::from_str(
                "0x035d56bac3f47246c5eed0e6642ca40dc262f9144b582f058bc23ded72aa72fa",
            )
            .unwrap(),
            logs_bloom: Bloom::from([0; 256]),
            difficulty: U256::zero(),
            number: 1,
            gas_limit: 0x016345785d8a0000,
            gas_used: 0xa8de,
            timestamp: 0x03e8,
            extra_data: Bytes::new(),
            prev_randao: H256::zero(),
            nonce: 0x0000000000000000,
            base_fee_per_gas: 0x07,
            withdrawals_root: H256::from_str(
                "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            )
            .unwrap(),
            blob_gas_used: 0x00,
            excess_blob_gas: 0x00,
            parent_beacon_block_root: H256::zero(),
        };
        let block_body = BlockBody {
            transactions: vec![Transaction::decode(&hex::decode("02f86c8330182480114e82f618946177843db3138ae69679a54b95cf345ed759450d870aa87bee53800080c080a0151ccc02146b9b11adf516e6787b59acae3e76544fdcd75e77e67c6b598ce65da064c5dd5aae2fbb535830ebbdad0234975cd7ece3562013b63ea18cc0df6c97d4").unwrap()).unwrap(),
            Transaction::decode(&hex::decode("f86d80843baa0c4082f618946177843db3138ae69679a54b95cf345ed759450d870aa87bee538000808360306ba0151ccc02146b9b11adf516e6787b59acae3e76544fdcd75e77e67c6b598ce65da064c5dd5aae2fbb535830ebbdad0234975cd7ece3562013b63ea18cc0df6c97d4").unwrap()).unwrap()],
            ommers: Default::default(),
            withdrawals: Default::default(),
        };
        (block_header, block_body)
    }

    fn test_store_block_number(store: Store) {
        let block_hash = H256::random();
        let block_number = 6;

        store.add_block_number(block_hash, block_number).unwrap();

        let stored_number = store.get_block_number(block_hash).unwrap().unwrap();

        assert_eq!(stored_number, block_number);
    }

    fn test_store_account_code(store: Store) {
        let code_hash = H256::random();
        let code = Bytes::from("kiwi");

        store.add_account_code(code_hash, code.clone()).unwrap();

        let stored_code = store.get_account_code(code_hash).unwrap().unwrap();

        assert_eq!(stored_code, code);
    }

    fn test_store_account_storage(store: Store) {
        let address = Address::random();
        let storage_key_a = H256::random();
        let storage_key_b = H256::random();
        let storage_value_a = H256::random();
        let storage_value_b = H256::random();

        store.add_storage_at(address, storage_key_a, storage_value_a).unwrap();
        store.add_storage_at(address, storage_key_b, storage_value_b).unwrap();

        let stored_value_a = store.get_storage_at(address, storage_key_a).unwrap().unwrap();
        let stored_value_b = store.get_storage_at(address, storage_key_b).unwrap().unwrap();

        assert_eq!(stored_value_a, storage_value_a);
        assert_eq!(stored_value_b, storage_value_b);
    }
}
