mod errors;
mod execution_result;

use crate::types::TxKind;

use super::{
    types::{Account, BlockHeader, Transaction},
    Address,
};
use revm::{
    inspector_handle_register,
    inspectors::TracerEip3155,
    primitives::{BlockEnv, Bytecode, TxEnv, U256},
    CacheState, Evm,
};
use std::collections::HashMap;
// Rename imported types for clarity
use revm::primitives::AccountInfo as RevmAccountInfo;
use revm::primitives::Address as RevmAddress;
use revm::primitives::TxKind as RevmTxKind;
// Export needed types
pub use errors::EvmError;
pub use execution_result::*;
pub use revm::primitives::SpecId;

pub fn execute_tx(
    tx: &Transaction,
    header: &BlockHeader,
    pre: &HashMap<Address, Account>, // TODO: Modify this type when we have a defined State structure
    spec_id: SpecId,
) -> Result<ExecutionResult, EvmError> {
    let block_env = block_env(header);
    let tx_env = tx_env(tx);
    let cache_state = cache_state(pre);
    let mut state = revm::db::State::builder()
        .with_cached_prestate(cache_state)
        .with_bundle_update()
        .build();
    let mut evm = Evm::builder()
        .with_db(&mut state)
        .with_block_env(block_env)
        .with_tx_env(tx_env)
        .with_spec_id(spec_id)
        .reset_handler()
        .with_external_context(TracerEip3155::new(Box::new(std::io::stderr())).without_summary())
        .append_handler_register(inspector_handle_register)
        .build();
    evm.preverify_transaction().map_err(EvmError::from)?;
    let tx_result = evm.transact().map_err(EvmError::from)?;
    Ok(tx_result.result.into())
}

fn cache_state(pre: &HashMap<Address, Account>) -> CacheState {
    let mut cache_state = revm::CacheState::new(false);
    for (address, account) in pre {
        let acc_info = RevmAccountInfo {
            balance: U256::from_limbs(account.info.balance.0),
            code_hash: account.info.code_hash.0.into(),
            code: Some(Bytecode::new_raw(account.code.clone().into())),
            nonce: account.info.nonce,
        };

        let mut storage = HashMap::new();
        for (k, v) in &account.storage {
            storage.insert(U256::from_be_bytes(k.0), U256::from_be_bytes(v.0));
        }

        cache_state.insert_account_with_storage(address.to_fixed_bytes().into(), acc_info, storage);
    }
    cache_state
}

fn block_env(header: &BlockHeader) -> BlockEnv {
    BlockEnv {
        number: U256::from(header.number),
        coinbase: RevmAddress(header.coinbase.0.into()),
        timestamp: U256::from(header.timestamp),
        gas_limit: U256::from(header.gas_limit),
        basefee: U256::from(header.base_fee_per_gas),
        difficulty: U256::from_limbs(header.difficulty.0),
        prevrandao: Some(header.prev_randao.as_fixed_bytes().into()),
        ..Default::default()
    }
}

fn tx_env(tx: &Transaction) -> TxEnv {
    TxEnv {
        caller: RevmAddress(tx.sender().0.into()),
        gas_limit: tx.gas_limit(),
        gas_price: U256::from(tx.gas_price()),
        transact_to: tx.to().into(),
        value: U256::from_limbs(tx.value().0),
        data: tx.data().clone().into(),
        nonce: Some(tx.nonce()),
        chain_id: tx.chain_id(),
        access_list: tx
            .access_list()
            .into_iter()
            .map(|(addr, list)| {
                (
                    RevmAddress(addr.0.into()),
                    list.into_iter().map(|a| U256::from_be_bytes(a.0)).collect(),
                )
            })
            .collect(),
        gas_priority_fee: tx.max_priority_fee().map(U256::from),
        ..Default::default()
    }
}

impl From<TxKind> for RevmTxKind {
    fn from(val: TxKind) -> Self {
        match val {
            TxKind::Call(address) => RevmTxKind::Call(address.0.into()),
            TxKind::Create => RevmTxKind::Create,
        }
    }
}
