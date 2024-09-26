use ethereum_types::{Address, H256, U256};

#[derive(Clone, Debug, Default)]
pub struct BlockEnv {
    /// The number of ancestor blocks of this block (block height).
    pub block_number: U256,
    /// Coinbase or miner or address that created and signed the block.
    ///
    /// This is the receiver address of all the gas spent in the block.
    pub coinbase: Address,
    /// The timestamp of the block in seconds since the UNIX epoch.
    pub timestamp: U256,
    //
    // The base fee per gas, added in the London upgrade with [EIP-1559].
    //
    // [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub base_fee_per_gas: U256,
    // Based on the python implementation, it's the gas limit of the block
    // https://github.com/ethereum/execution-specs/blob/master/src/ethereum/cancun/blocks.py
    pub gas_limit: usize,
    // Chain ID of the EVM, it will be compared to the transaction's Chain ID.
    // Chain ID is introduced here https://eips.ethereum.org/EIPS/eip-155
    pub chain_id: usize,
    // The difficulty of the block.
    //
    // Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    //pub difficulty: U256,
    // The output of the randomness beacon provided by the beacon chain.
    //
    // Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    //
    // NOTE: `prevrandao` can be found in a block in place of `mix_hash`.
    //
    // [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    pub prevrandao: Option<H256>,
    // Excess blob gas and blob gasprice.
    //
    // Incorporated as part of the Cancun upgrade via [EIP-4844].
    //
    // [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub excess_blob_gas: Option<u64>,
}
pub const MIN_BLOB_GASPRICE: u64 = 1;
pub const BLOB_GASPRICE_UPDATE_FRACTION: u64 = 3338477;
impl BlockEnv {
    /// Calculates the blob gas price from the header's excess blob gas field.
    ///
    /// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
    /// (`get_blob_gasprice`).
    pub fn calculate_blob_gas_price(&self) -> U256 {
        U256::from(fake_exponential(
            MIN_BLOB_GASPRICE,
            self.excess_blob_gas.unwrap_or_default(),
            BLOB_GASPRICE_UPDATE_FRACTION,
        ))
    }
}

/// Approximates `factor * e ** (numerator / denominator)` using Taylor expansion.
///
/// This is used to calculate the blob price.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers)
/// (`fake_exponential`).
///
/// # Panics
///
/// This function panics if `denominator` is zero.
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u128 {
    assert_ne!(denominator, 0, "attempt to divide by zero");
    let factor = factor as u128;
    let numerator = numerator as u128;
    let denominator = denominator as u128;

    let mut i = 1;
    let mut output = 0;
    let mut numerator_accum = factor * denominator;
    while numerator_accum > 0 {
        output += numerator_accum;

        // Denominator is asserted as not zero at the start of the function.
        numerator_accum = (numerator_accum * numerator) / (denominator * i);
        i += 1;
    }
    output / denominator
}
