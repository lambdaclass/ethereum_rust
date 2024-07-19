use ethereum_rust_core::{Address as CoreAddress, H256 as CoreH256};
use ethereum_rust_storage::{error::StoreError, Store};
use revm::primitives::{
    AccountInfo as RevmAccountInfo, Address as RevmAddress, Bytecode as RevmBytecode,
    Bytes as RevmBytes, B256 as RevmB256, U256 as RevmU256,
};

pub struct StoreWrapper(pub Store);

impl revm::Database for StoreWrapper {
    #[doc = " The database error type."]
    type Error = StoreError;

    #[doc = " Get basic account information."]
    fn basic(&mut self, address: RevmAddress) -> Result<Option<RevmAccountInfo>, Self::Error> {
        let acc_info = match self
            .0
            .get_account_info(CoreAddress::from(address.0.as_ref()))?
        {
            None => return Ok(None),
            Some(acc_info) => acc_info,
        };
        let code = self
            .0
            .get_account_code(acc_info.code_hash)?
            .map(|b| RevmBytecode::new_raw(RevmBytes(b)));

        Ok(Some(RevmAccountInfo {
            balance: RevmU256::from_limbs(acc_info.balance.0),
            nonce: acc_info.nonce,
            code_hash: RevmB256::from(acc_info.code_hash.0),
            code,
        }))
    }

    #[doc = " Get account code by its hash."]
    fn code_by_hash(&mut self, code_hash: RevmB256) -> Result<RevmBytecode, Self::Error> {
        self.0
            .get_account_code(CoreH256::from(code_hash.as_ref()))?
            .map(|b| RevmBytecode::new_raw(RevmBytes(b)))
            .ok_or_else(|| StoreError::Custom(format!("No code for hash {code_hash}")))
    }

    #[doc = " Get storage value of address at index."]
    fn storage(&mut self, address: RevmAddress, index: RevmU256) -> Result<RevmU256, Self::Error> {
        self.0
            .get_storage_at(
                CoreAddress::from(address.0.as_ref()),
                CoreH256::from(index.to_be_bytes()),
            )?
            .map(|value| RevmU256::from_be_bytes(value.0))
            .ok_or_else(|| {
                StoreError::Custom(format!(
                    "No storage value found for address: {address}, key: {index}"
                ))
            })
    }

    #[doc = " Get block hash by block number."]
    fn block_hash(&mut self, number: RevmU256) -> Result<RevmB256, Self::Error> {
        self.0
            .get_block_header(number.to())?
            .map(|header| RevmB256::from_slice(&header.compute_block_hash().0))
            .ok_or_else(|| StoreError::Custom(format!("Block {number} not found")))
    }
}
