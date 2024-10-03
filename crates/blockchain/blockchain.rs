pub mod constants;
pub mod error;
pub mod mempool;
pub mod payload;
mod smoke_test;

use constants::{GAS_PER_BLOB, MAX_BLOB_GAS_PER_BLOCK, MAX_BLOB_NUMBER_PER_BLOCK};
use error::{ChainError, InvalidBlockError, InvalidForkChoice};
use ethereum_rust_core::types::{
    validate_block_header, validate_cancun_header_fields, validate_no_cancun_header_fields, Block,
    BlockHash, BlockHeader, BlockNumber, EIP4844Transaction, Receipt, Transaction,
};
use ethereum_rust_core::{H256, U256};

use ethereum_rust_evm::{
    evm_state, execute_block, get_state_transitions, spec_id, EvmState, SpecId,
};
use ethereum_rust_storage::error::StoreError;
use ethereum_rust_storage::Store;

//TODO: Implement a struct Chain or BlockChain to encapsulate
//functionality and canonical chain state and config

/// Adds a new block to the store. It may or may not be canonical, as long as its ancestry links
/// with the canonical chain and its parent's post-state is calculated. It doesn't modify the
/// canonical chain/head. Fork choice needs to be updated for that in a separate step.
///
/// Performs pre and post execution validation, and updates the database with the post state.
pub fn add_block(block: &Block, storage: &Store) -> Result<(), ChainError> {
    // TODO(#438): handle cases where blocks are missing between the canonical chain and the block.

    // Validate if it can be the new head and find the parent
    let parent_header = find_parent_header(&block.header, storage)?;
    let mut state = evm_state(storage.clone(), block.header.parent_hash);

    // Validate the block pre-execution
    validate_block(block, &parent_header, &state)?;

    let receipts = execute_block(block, &mut state)?;

    validate_gas_used(&receipts, &block.header)?;

    let account_updates = get_state_transitions(&mut state);

    // Apply the account updates over the last block's state and compute the new state root
    let new_state_root = state
        .database()
        .apply_account_updates(block.header.parent_hash, &account_updates)?
        .unwrap_or_default();

    // Check state root matches the one in block header after execution
    validate_state_root(&block.header, new_state_root)?;

    let block_hash = block.header.compute_block_hash();
    store_block(storage, block.clone())?;
    store_receipts(storage, receipts, block_hash)?;

    Ok(())
}

/// Stores block and header in the database
pub fn store_block(storage: &Store, block: Block) -> Result<(), ChainError> {
    storage.add_block(block)?;
    Ok(())
}

pub fn store_receipts(
    storage: &Store,
    receipts: Vec<Receipt>,
    block_hash: BlockHash,
) -> Result<(), ChainError> {
    for (index, receipt) in receipts.into_iter().enumerate() {
        storage.add_receipt(block_hash, index as u64, receipt)?;
    }
    Ok(())
}

/// Performs post-execution checks
pub fn validate_state_root(
    block_header: &BlockHeader,
    new_state_root: H256,
) -> Result<(), ChainError> {
    // Compare state root
    if new_state_root == block_header.state_root {
        Ok(())
    } else {
        Err(ChainError::InvalidBlock(
            InvalidBlockError::StateRootMismatch,
        ))
    }
}

pub fn latest_valid_hash(storage: &Store) -> Result<H256, ChainError> {
    if let Some(latest_block_number) = storage.get_latest_block_number()? {
        if let Some(latest_valid_header) = storage.get_block_header(latest_block_number)? {
            let latest_valid_hash = latest_valid_header.compute_block_hash();
            return Ok(latest_valid_hash);
        }
    }
    Err(ChainError::StoreError(StoreError::Custom(
        "Could not find latest valid hash".to_string(),
    )))
}

/// Validates if the provided block could be the new head of the chain, and returns the
/// parent_header in that case
pub fn find_parent_header(
    block_header: &BlockHeader,
    storage: &Store,
) -> Result<BlockHeader, ChainError> {
    let Some(parent_header) = storage.get_block_header_by_hash(block_header.parent_hash)? else {
        return Err(ChainError::ParentNotFound);
    };

    if parent_header.number == block_header.number - 1 {
        Ok(parent_header)
    } else {
        Err(ChainError::ParentNotFound)
    }
}

/// Performs pre-execution validation of the block's header values in reference to the parent_header
/// Verifies that blob gas fields in the header are correct in reference to the block's body.
/// If a block passes this check, execution will still fail with execute_block when a transaction runs out of gas
pub fn validate_block(
    block: &Block,
    parent_header: &BlockHeader,
    state: &EvmState,
) -> Result<(), ChainError> {
    let spec = spec_id(state.database(), block.header.timestamp).unwrap();

    // Verify initial header validity against parent
    let mut valid_header = validate_block_header(&block.header, parent_header);

    valid_header = match spec {
        SpecId::CANCUN => {
            valid_header && validate_cancun_header_fields(&block.header, parent_header)
        }
        _ => valid_header && validate_no_cancun_header_fields(&block.header),
    };
    if !valid_header {
        return Err(ChainError::InvalidBlock(InvalidBlockError::InvalidHeader));
    }

    if spec == SpecId::CANCUN {
        verify_blob_gas_usage(block)?
    }
    Ok(())
}

pub fn is_canonical(
    store: &Store,
    block_number: BlockNumber,
    block_hash: BlockHash,
) -> Result<bool, StoreError> {
    match store.get_canonical_block_hash(block_number)? {
        Some(hash) if hash == block_hash => Ok(true),
        _ => Ok(false),
    }
}

pub fn new_head(
    store: &Store,
    head_hash: H256,
    safe_hash: H256,
    finalized_hash: H256,
) -> Result<(), InvalidForkChoice> {
    if head_hash.is_zero() {
        return Err(InvalidForkChoice::InvalidHeadHash);
    }

    // We get the block bodies even if we only use headers them so we check that they are
    // stored too.
    let finalized_header_res = store.get_block_by_hash(finalized_hash).map_err(wrap)?;
    let safe_header_res = store.get_block_by_hash(safe_hash).map_err(wrap)?;
    let head_header_res = store.get_block_by_hash(head_hash).map_err(wrap)?;

    // Check that we already have all the needed blocks stored and that we have the ancestors
    // if we have the descendants, as we are working on the assumption that we only add block
    // if they are connected to the canonical chain.
    let (finalized, safe, head) = match (finalized_header_res, safe_header_res, head_header_res) {
        (None, Some(_), _) => return Err(InvalidForkChoice::ElementNotFound),
        (_, None, Some(_)) => return Err(InvalidForkChoice::ElementNotFound),
        (Some(f), Some(s), Some(h)) => (f.header, s.header, h.header),
        _ => return Err(InvalidForkChoice::Syncing),
    };

    // Check that we are not being pushed pre-merge
    total_difficulty_check(&head_hash, &head, &store)?;

    // Check that the headers are in the correct order.
    if finalized.number > safe.number || safe.number > head.number {
        return Err(InvalidForkChoice::Unordered);
    }

    // If the head block is already in our canonical chain, the beacon client is
    // probably resyncing. Ignore the update.
    if is_canonical(&store, head.number, head_hash).map_err(wrap)? {
        return Ok(());
    }

    // If both finalized and safe blocks are canonical, we can skip the ancestry check.
    let finalized_canonical =
        is_canonical(&store, finalized.number, finalized_hash).map_err(wrap)?;
    let safe_canonical = is_canonical(&store, safe.number, safe_hash).map_err(wrap)?;

    // Find out if blocks are correctly connected.
    let Some(head_ancestry) = find_ancestry(&store, &safe, &head).map_err(wrap)? else {
        return Err(InvalidForkChoice::Disconnected(
            error::ForkChoiceElement::Head,
            error::ForkChoiceElement::Safe,
        ));
    };

    let safe_ancestry = if safe_canonical && finalized_canonical {
        // Skip check. We will not canonize anything between safe and finalized blocks.
        Vec::new()
    } else {
        let Some(ancestry) = find_ancestry(&store, &finalized, &safe).map_err(wrap)? else {
            return Err(InvalidForkChoice::Disconnected(
                error::ForkChoiceElement::Safe,
                error::ForkChoiceElement::Finalized,
            ));
        };
        ancestry
    };

    // Canonize blocks from both ancestries.
    for (number, hash) in safe_ancestry {
        store.set_canonical_block(number, hash).map_err(wrap)?;
    }

    for (number, hash) in head_ancestry {
        store.set_canonical_block(number, hash).map_err(wrap)?;
    }

    store
        .set_canonical_block(head.number, head_hash)
        .map_err(wrap)?;
    store
        .set_canonical_block(safe.number, safe_hash)
        .map_err(wrap)?;
    store
        .set_canonical_block(finalized.number, finalized_hash)
        .map_err(wrap)?;

    store
        .update_finalized_block_number(finalized.number)
        .map_err(wrap)?;
    store.update_safe_block_number(safe.number).map_err(wrap)?;
    Ok(())
}

fn validate_gas_used(receipts: &[Receipt], block_header: &BlockHeader) -> Result<(), ChainError> {
    if let Some(last) = receipts.last() {
        if last.cumulative_gas_used != block_header.gas_used {
            return Err(ChainError::InvalidBlock(InvalidBlockError::GasUsedMismatch));
        }
    }
    Ok(())
}

// Wrap store errors inside invalid fork choice errors.
fn wrap(se: StoreError) -> InvalidForkChoice {
    InvalidForkChoice::StoreError(se)
}

fn verify_blob_gas_usage(block: &Block) -> Result<(), ChainError> {
    let mut blob_gas_used = 0_u64;
    let mut blobs_in_block = 0_u64;
    for transaction in block.body.transactions.iter() {
        if let Transaction::EIP4844Transaction(tx) = transaction {
            blob_gas_used += get_total_blob_gas(tx);
            blobs_in_block += tx.blob_versioned_hashes.len() as u64;
        }
    }
    if blob_gas_used > MAX_BLOB_GAS_PER_BLOCK {
        return Err(ChainError::InvalidBlock(
            InvalidBlockError::ExceededMaxBlobGasPerBlock,
        ));
    }
    if blobs_in_block > MAX_BLOB_NUMBER_PER_BLOCK {
        return Err(ChainError::InvalidBlock(
            InvalidBlockError::ExceededMaxBlobNumberPerBlock,
        ));
    }
    if block
        .header
        .blob_gas_used
        .is_some_and(|header_blob_gas_used| header_blob_gas_used != blob_gas_used)
    {
        return Err(ChainError::InvalidBlock(
            InvalidBlockError::BlobGasUsedMismatch,
        ));
    }
    Ok(())
}

/// Calculates the blob gas required by a transaction
fn get_total_blob_gas(tx: &EIP4844Transaction) -> u64 {
    GAS_PER_BLOB * tx.blob_versioned_hashes.len() as u64
}

fn total_difficulty_check<'a>(
    head_block_hash: &'a H256,
    head_block: &'a BlockHeader,
    storage: &'a Store,
) -> Result<(), InvalidForkChoice> {
    if !head_block.difficulty.is_zero() || head_block.number == 0 {
        let total_difficulty = storage
            .get_block_total_difficulty(*head_block_hash)
            .map_err(wrap)?;
        let parent_total_difficulty = storage
            .get_block_total_difficulty(head_block.parent_hash)
            .map_err(wrap)?;
        let terminal_total_difficulty = storage
            .get_chain_config()
            .map_err(wrap)?
            .terminal_total_difficulty;
        if terminal_total_difficulty.is_none()
            || total_difficulty.is_none()
            || head_block.number > 0 && parent_total_difficulty.is_none()
        {
            return Err(InvalidForkChoice::StoreError(StoreError::Custom(
                "Total difficulties unavailable for terminal total difficulty check".to_string(),
            )));
        }
        if total_difficulty.unwrap() < terminal_total_difficulty.unwrap().into() {
            return Err(InvalidForkChoice::PreMergeBlock);
        }
        if head_block.number > 0
            && parent_total_difficulty.unwrap() >= terminal_total_difficulty.unwrap().into()
        {
            return Err(InvalidForkChoice::StoreError(StoreError::Custom(
                "Parent block is already post terminal total difficulty".to_string(),
            )));
        }
    }
    Ok(())
}

// Find branch of the blockchain connecting two blocks. If the blocks are connected through
// parent hashes, then a vector of number-hash pairs is returned for the branch. If they are not
// connected, an error is returned.
//
// Return values:
// - Err(StoreError): a db-related error happened.
// - Ok(None): the headers are not related by ancestry.
// - Ok(Some([])): the headers are the same block.
// - Ok(Some(branch)): the "branch" is a sequence of blocks that connects the ancestor and the
//   descendant.
fn find_ancestry(
    storage: &Store,
    ancestor: &BlockHeader,
    descendant: &BlockHeader,
) -> Result<Option<Vec<(BlockNumber, BlockHash)>>, StoreError> {
    let mut block_number = descendant.number;
    let mut found = false;
    let descendant_hash = descendant.compute_block_hash();
    let ancestor_hash = ancestor.compute_block_hash();
    let mut header = descendant.clone();
    let mut branch = Vec::new();

    if ancestor.number == descendant.number {
        if ancestor_hash == descendant_hash {
            return Ok(Some(branch));
        } else {
            return Ok(None);
        }
    }

    println!(
        "Block numbers: ancestor: {}. descendant: {}",
        ancestor.number, descendant.number
    );

    while block_number > ancestor.number && !found {
        block_number -= 1;
        let parent_hash = header.parent_hash;

        // Check that the parent exists.
        let parent_header = match storage.get_block_header_by_hash(parent_hash) {
            Ok(Some(header)) => header,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };

        if block_number == ancestor.number {
            if ancestor_hash == parent_hash {
                found = true;
            } else {
                return Ok(None);
            }
        } else {
            branch.push((block_number, parent_hash));
        }

        header = parent_header;
    }

    if found {
        Ok(Some(branch))
    } else {
        Ok(None)
    }
}
#[cfg(test)]
mod tests {}
