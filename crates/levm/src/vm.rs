use std::collections::HashMap;

use crate::{
    call_frame::CallFrame,
    constants::*,
    opcodes::Opcode,
    primitives::{Address, Bytes, U256, U512},
};
use sha3::{Digest, Keccak256};
use std::i64;

#[derive(Clone, Default, Debug)]
pub struct Account {
    balance: U256,
    bytecode: Bytes,
}

impl Account {
    pub fn new(balance: U256, bytecode: Bytes) -> Self {
        Self { balance, bytecode }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VM {
    call_frames: Vec<CallFrame>,
    pub accounts: HashMap<Address, Account>, // change to Address
    gas_limit: u64,
    pub consumed_gas: u64, // TODO: check where to place these two in the future, probably TxEnv
}

/// Shifts the value to the right by 255 bits and checks the most significant bit is a 1
fn is_negative(value: U256) -> bool {
    value.bit(255)
}
/// negates a number in two's complement
fn negate(value: U256) -> U256 {
    !value + U256::one()
}

impl VM {
    pub fn new(bytecode: Bytes, address: Address, balance: U256) -> Self {
        let initial_account = Account::new(balance, bytecode.clone());

        let initial_call_frame = CallFrame::new(bytecode);
        let mut accounts = HashMap::new();
        accounts.insert(address, initial_account);
        Self {
            call_frames: vec![initial_call_frame.clone()],
            accounts,
            gas_limit: i64::MAX as _, // it is initialized like this for testing
            consumed_gas: TX_BASE_COST,
        }
    }

    pub fn execute(&mut self) {
        let mut tx_env = self.clone(); // simulates a TxEnv
        let mut current_call_frame = self.call_frames.pop().unwrap();
        loop {
            match current_call_frame.next_opcode().unwrap() {
                Opcode::STOP => break,
                Opcode::ADD => {
                    if tx_env.consumed_gas + gas_cost::ADD > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let augend = current_call_frame.stack.pop().unwrap();
                    let addend = current_call_frame.stack.pop().unwrap();
                    let sum = augend.overflowing_add(addend).0;
                    current_call_frame.stack.push(sum);
                    tx_env.consumed_gas += gas_cost::ADD
                }
                Opcode::MUL => {
                    if tx_env.consumed_gas + gas_cost::MUL > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let multiplicand = current_call_frame.stack.pop().unwrap();
                    let multiplier = current_call_frame.stack.pop().unwrap();
                    let product = multiplicand.overflowing_mul(multiplier).0;
                    current_call_frame.stack.push(product);
                    tx_env.consumed_gas += gas_cost::MUL
                }
                Opcode::SUB => {
                    if tx_env.consumed_gas + gas_cost::SUB > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let minuend = current_call_frame.stack.pop().unwrap();
                    let subtrahend = current_call_frame.stack.pop().unwrap();
                    let difference = minuend.overflowing_sub(subtrahend).0;
                    current_call_frame.stack.push(difference);
                    tx_env.consumed_gas += gas_cost::SUB
                }
                Opcode::DIV => {
                    if tx_env.consumed_gas + gas_cost::DIV > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let dividend = current_call_frame.stack.pop().unwrap();
                    let divisor = current_call_frame.stack.pop().unwrap();
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }
                    let quotient = dividend / divisor;
                    current_call_frame.stack.push(quotient);
                    tx_env.consumed_gas += gas_cost::DIV
                }
                Opcode::SDIV => {
                    if tx_env.consumed_gas + gas_cost::SDIV > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let dividend = current_call_frame.stack.pop().unwrap();
                    let divisor = current_call_frame.stack.pop().unwrap();
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }

                    let dividend_is_negative = is_negative(dividend);
                    let divisor_is_negative = is_negative(divisor);
                    let dividend = if dividend_is_negative {
                        negate(dividend)
                    } else {
                        dividend
                    };
                    let divisor = if divisor_is_negative {
                        negate(divisor)
                    } else {
                        divisor
                    };
                    let quotient = dividend / divisor;
                    let quotient_is_negative = dividend_is_negative ^ divisor_is_negative;
                    let quotient = if quotient_is_negative {
                        negate(quotient)
                    } else {
                        quotient
                    };

                    current_call_frame.stack.push(quotient);
                    tx_env.consumed_gas += gas_cost::SDIV
                }
                Opcode::MOD => {
                    if tx_env.consumed_gas + gas_cost::MOD > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let dividend = current_call_frame.stack.pop().unwrap();
                    let divisor = current_call_frame.stack.pop().unwrap();
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }
                    let remainder = dividend % divisor;
                    current_call_frame.stack.push(remainder);
                    tx_env.consumed_gas += gas_cost::MOD
                }
                Opcode::SMOD => {
                    if tx_env.consumed_gas + gas_cost::SMOD > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let dividend = current_call_frame.stack.pop().unwrap();
                    let divisor = current_call_frame.stack.pop().unwrap();
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }

                    let dividend_is_negative = is_negative(dividend);
                    let divisor_is_negative = is_negative(divisor);
                    let dividend = if dividend_is_negative {
                        negate(dividend)
                    } else {
                        dividend
                    };
                    let divisor = if divisor_is_negative {
                        negate(divisor)
                    } else {
                        divisor
                    };
                    let remainder = dividend % divisor;
                    let remainder_is_negative = dividend_is_negative ^ divisor_is_negative;
                    let remainder = if remainder_is_negative {
                        negate(remainder)
                    } else {
                        remainder
                    };

                    current_call_frame.stack.push(remainder);
                    tx_env.consumed_gas += gas_cost::MOD
                }
                Opcode::ADDMOD => {
                    if tx_env.consumed_gas + gas_cost::ADDMOD > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let augend = current_call_frame.stack.pop().unwrap();
                    let addend = current_call_frame.stack.pop().unwrap();
                    let divisor = current_call_frame.stack.pop().unwrap();
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }
                    let (sum, overflow) = augend.overflowing_add(addend);
                    let mut remainder = sum % divisor;
                    if overflow || remainder > divisor {
                        remainder = remainder.overflowing_sub(divisor).0;
                    }

                    current_call_frame.stack.push(remainder);
                    tx_env.consumed_gas += gas_cost::ADDMOD
                }
                Opcode::MULMOD => {
                    if tx_env.consumed_gas + gas_cost::MULMOD > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let multiplicand = U512::from(current_call_frame.stack.pop().unwrap());
                    let multiplier = U512::from(current_call_frame.stack.pop().unwrap());
                    let divisor = U512::from(current_call_frame.stack.pop().unwrap());
                    if divisor.is_zero() {
                        current_call_frame.stack.push(U256::zero());
                        continue;
                    }

                    let (product, overflow) = multiplicand.overflowing_mul(multiplier);
                    let mut remainder = product % divisor;
                    if overflow || remainder > divisor {
                        remainder = remainder.overflowing_sub(divisor).0;
                    }
                    let mut result = Vec::new();
                    for byte in remainder.0.iter().take(4) {
                        let bytes = byte.to_le_bytes();
                        result.extend_from_slice(&bytes);
                    }
                    // before reverse we have something like [120, 255, 0, 0....]
                    // after reverse we get the [0, 0, ...., 255, 120] which is the correct order for the little endian u256
                    result.reverse();
                    let remainder = U256::from(result.as_slice());
                    current_call_frame.stack.push(remainder);
                    tx_env.consumed_gas += gas_cost::MULMOD
                }
                Opcode::EXP => {
                    let base = current_call_frame.stack.pop().unwrap();
                    let exponent = current_call_frame.stack.pop().unwrap();

                    let exponent_byte_size = (exponent.bits() as u64 + 7) / 8;
                    let gas_cost =
                        gas_cost::EXP_STATIC + gas_cost::EXP_DYNAMIC_BASE * exponent_byte_size;
                    if tx_env.consumed_gas + gas_cost > tx_env.gas_limit {
                        break; // should revert the tx
                    }

                    let power = base.overflowing_pow(exponent).0;
                    current_call_frame.stack.push(power);
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::SIGNEXTEND => {
                    if tx_env.consumed_gas + gas_cost::SIGNEXTEND > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let byte_size = current_call_frame.stack.pop().unwrap();
                    let value_to_extend = current_call_frame.stack.pop().unwrap();

                    let bits_per_byte = U256::from(8);
                    let sign_bit_position_on_byte = 7;
                    let max_byte_size = 31;

                    let byte_size = byte_size.min(U256::from(max_byte_size));
                    let sign_bit_index = bits_per_byte * byte_size + sign_bit_position_on_byte;
                    let is_negative = value_to_extend.bit(sign_bit_index.as_usize());
                    let sign_bit_mask = (U256::one() << sign_bit_index) - U256::one();
                    let result = if is_negative {
                        value_to_extend | !sign_bit_mask
                    } else {
                        value_to_extend & sign_bit_mask
                    };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::SIGNEXTEND
                }
                Opcode::LT => {
                    if tx_env.consumed_gas + gas_cost::LT > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let lho = current_call_frame.stack.pop().unwrap();
                    let rho = current_call_frame.stack.pop().unwrap();
                    let result = if lho < rho { U256::one() } else { U256::zero() };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::LT
                }
                Opcode::GT => {
                    if tx_env.consumed_gas + gas_cost::GT > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let lho = current_call_frame.stack.pop().unwrap();
                    let rho = current_call_frame.stack.pop().unwrap();
                    let result = if lho > rho { U256::one() } else { U256::zero() };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::GT
                }
                Opcode::SLT => {
                    if tx_env.consumed_gas + gas_cost::SLT > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let lho = current_call_frame.stack.pop().unwrap();
                    let rho = current_call_frame.stack.pop().unwrap();
                    let lho_is_negative = lho.bit(255);
                    let rho_is_negative = rho.bit(255);
                    let result = if lho_is_negative == rho_is_negative {
                        // if both have the same sign, compare their magnitudes
                        if lho < rho {
                            U256::one()
                        } else {
                            U256::zero()
                        }
                    } else {
                        // if they have different signs, the negative number is smaller
                        if lho_is_negative {
                            U256::one()
                        } else {
                            U256::zero()
                        }
                    };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::SLT
                }
                Opcode::SGT => {
                    if tx_env.consumed_gas + gas_cost::SGT > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let lho = current_call_frame.stack.pop().unwrap();
                    let rho = current_call_frame.stack.pop().unwrap();
                    let lho_is_negative = lho.bit(255);
                    let rho_is_negative = rho.bit(255);
                    let result = if lho_is_negative == rho_is_negative {
                        // if both have the same sign, compare their magnitudes
                        if lho > rho {
                            U256::one()
                        } else {
                            U256::zero()
                        }
                    } else {
                        // if they have different signs, the positive number is bigger
                        if rho_is_negative {
                            U256::one()
                        } else {
                            U256::zero()
                        }
                    };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::SGT
                }
                Opcode::EQ => {
                    if tx_env.consumed_gas + gas_cost::EQ > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let lho = current_call_frame.stack.pop().unwrap();
                    let rho = current_call_frame.stack.pop().unwrap();
                    let result = if lho == rho {
                        U256::one()
                    } else {
                        U256::zero()
                    };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::EQ
                }
                Opcode::ISZERO => {
                    if tx_env.consumed_gas + gas_cost::ISZERO > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let operand = current_call_frame.stack.pop().unwrap();
                    let result = if operand == U256::zero() {
                        U256::one()
                    } else {
                        U256::zero()
                    };
                    current_call_frame.stack.push(result);
                    tx_env.consumed_gas += gas_cost::ISZERO
                }
                Opcode::KECCAK256 => {
                    let offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let size = current_call_frame.stack.pop().unwrap().try_into().unwrap();

                    let minimum_word_size = (size + WORD_SIZE - 1) / WORD_SIZE;
                    let memory_expansion_cost =
                        current_call_frame.memory.expansion_cost(offset + size);
                    let gas_cost = gas_cost::KECCAK25_STATIC
                        + gas_cost::KECCAK25_DYNAMIC_BASE * minimum_word_size as u64
                        + memory_expansion_cost;
                    if tx_env.consumed_gas + gas_cost > tx_env.gas_limit {
                        break; // should revert the tx
                    }

                    let value_bytes = current_call_frame.memory.load_range(offset, size);

                    let mut hasher = Keccak256::new();
                    hasher.update(value_bytes);
                    let result = hasher.finalize();
                    current_call_frame
                        .stack
                        .push(U256::from_big_endian(&result));
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::JUMP => {
                    if tx_env.consumed_gas + gas_cost::JUMP > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let jump_address = current_call_frame.stack.pop().unwrap();
                    current_call_frame.jump(jump_address);
                    tx_env.consumed_gas += gas_cost::JUMP
                }
                Opcode::JUMPI => {
                    if tx_env.consumed_gas + gas_cost::JUMPI > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let jump_address = current_call_frame.stack.pop().unwrap();
                    let condition = current_call_frame.stack.pop().unwrap();
                    if condition != U256::zero() {
                        current_call_frame.jump(jump_address);
                    }
                    tx_env.consumed_gas += gas_cost::JUMPI
                }
                Opcode::JUMPDEST => {
                    // just consume some gas, jumptable written at the start
                    if tx_env.consumed_gas + gas_cost::JUMPDEST > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    tx_env.consumed_gas += gas_cost::JUMPDEST
                }
                Opcode::PC => {
                    if tx_env.consumed_gas + gas_cost::PC > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    current_call_frame
                        .stack
                        .push(U256::from(current_call_frame.pc - 1));
                    tx_env.consumed_gas += gas_cost::PC
                }
                Opcode::PUSH0 => {
                    if tx_env.consumed_gas + gas_cost::PUSH0 > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    current_call_frame.stack.push(U256::zero());
                    tx_env.consumed_gas += gas_cost::PUSH0
                }
                // PUSHn
                op if (Opcode::PUSH1..Opcode::PUSH32).contains(&op) => {
                    if tx_env.consumed_gas + gas_cost::PUSHN > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let n_bytes = (op as u8) - (Opcode::PUSH1 as u8) + 1;
                    let next_n_bytes = current_call_frame
                        .bytecode
                        .get(current_call_frame.pc()..current_call_frame.pc() + n_bytes as usize)
                        .expect("invalid bytecode");
                    let value_to_push = U256::from(next_n_bytes);
                    current_call_frame.stack.push(value_to_push);
                    current_call_frame.increment_pc_by(n_bytes as usize);
                    tx_env.consumed_gas += gas_cost::PUSHN
                }
                Opcode::PUSH32 => {
                    if tx_env.consumed_gas + gas_cost::PUSHN > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let next_32_bytes = current_call_frame
                        .bytecode
                        .get(current_call_frame.pc()..current_call_frame.pc() + WORD_SIZE)
                        .unwrap();
                    let value_to_push = U256::from(next_32_bytes);
                    current_call_frame.stack.push(value_to_push);
                    current_call_frame.increment_pc_by(WORD_SIZE);
                    tx_env.consumed_gas += gas_cost::PUSHN
                }
                Opcode::AND => {
                    if tx_env.consumed_gas + gas_cost::AND > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let a = current_call_frame.stack.pop().unwrap();
                    let b = current_call_frame.stack.pop().unwrap();
                    current_call_frame.stack.push(a & b);
                    tx_env.consumed_gas += gas_cost::AND
                }
                Opcode::OR => {
                    if tx_env.consumed_gas + gas_cost::OR > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let a = current_call_frame.stack.pop().unwrap();
                    let b = current_call_frame.stack.pop().unwrap();
                    current_call_frame.stack.push(a | b);
                    tx_env.consumed_gas += gas_cost::OR
                }
                Opcode::XOR => {
                    if tx_env.consumed_gas + gas_cost::XOR > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let a = current_call_frame.stack.pop().unwrap();
                    let b = current_call_frame.stack.pop().unwrap();
                    current_call_frame.stack.push(a ^ b);
                    tx_env.consumed_gas += gas_cost::XOR
                }
                Opcode::NOT => {
                    if tx_env.consumed_gas + gas_cost::NOT > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let a = current_call_frame.stack.pop().unwrap();
                    current_call_frame.stack.push(!a);
                    tx_env.consumed_gas += gas_cost::NOT
                }
                Opcode::BYTE => {
                    if tx_env.consumed_gas + gas_cost::BYTE > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let op1 = current_call_frame.stack.pop().unwrap();
                    let op2 = current_call_frame.stack.pop().unwrap();

                    let byte_index = op1.try_into().unwrap_or(usize::MAX);

                    if byte_index < WORD_SIZE {
                        current_call_frame
                            .stack
                            .push(U256::from(op2.byte(WORD_SIZE - 1 - byte_index)));
                    } else {
                        current_call_frame.stack.push(U256::zero());
                    }
                    tx_env.consumed_gas += gas_cost::BYTE
                }
                Opcode::SHL => {
                    if tx_env.consumed_gas + gas_cost::SHL > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let shift = current_call_frame.stack.pop().unwrap();
                    let value = current_call_frame.stack.pop().unwrap();
                    if shift < U256::from(256) {
                        current_call_frame.stack.push(value << shift);
                    } else {
                        current_call_frame.stack.push(U256::zero());
                    }
                    tx_env.consumed_gas += gas_cost::SHL
                }
                Opcode::SHR => {
                    if tx_env.consumed_gas + gas_cost::SHR > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let shift = current_call_frame.stack.pop().unwrap();
                    let value = current_call_frame.stack.pop().unwrap();
                    if shift < U256::from(256) {
                        current_call_frame.stack.push(value >> shift);
                    } else {
                        current_call_frame.stack.push(U256::zero());
                    }
                    tx_env.consumed_gas += gas_cost::SHR
                }
                Opcode::SAR => {
                    if tx_env.consumed_gas + gas_cost::SAR > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let shift = current_call_frame.stack.pop().unwrap();
                    let value = current_call_frame.stack.pop().unwrap();
                    let res = if shift < U256::from(256) {
                        arithmetic_shift_right(value, shift)
                    } else if value.bit(255) {
                        U256::MAX
                    } else {
                        U256::zero()
                    };
                    current_call_frame.stack.push(res);
                    tx_env.consumed_gas += gas_cost::SAR
                }
                // DUPn
                op if (Opcode::DUP1..=Opcode::DUP16).contains(&op) => {
                    if tx_env.consumed_gas + gas_cost::DUPN > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let depth = (op as u8) - (Opcode::DUP1 as u8) + 1;
                    assert!(
                        current_call_frame.stack.len().ge(&(depth as usize)),
                        "stack underflow: not enough values on the stack"
                    );
                    let value_at_depth = current_call_frame
                        .stack
                        .get(current_call_frame.stack.len() - depth as usize)
                        .unwrap();
                    current_call_frame.stack.push(*value_at_depth);
                    tx_env.consumed_gas += gas_cost::DUPN
                }
                // SWAPn
                op if (Opcode::SWAP1..=Opcode::SWAP16).contains(&op) => {
                    if tx_env.consumed_gas + gas_cost::SWAPN > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let depth = (op as u8) - (Opcode::SWAP1 as u8) + 1;
                    assert!(
                        current_call_frame.stack.len().ge(&(depth as usize)),
                        "stack underflow: not enough values on the stack"
                    );
                    let stack_top_index = current_call_frame.stack.len();
                    let to_swap_index = stack_top_index.checked_sub(depth as usize).unwrap();
                    current_call_frame
                        .stack
                        .swap(stack_top_index - 1, to_swap_index - 1);
                    tx_env.consumed_gas += gas_cost::SWAPN
                }
                Opcode::POP => {
                    if tx_env.consumed_gas + gas_cost::POP > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    current_call_frame.stack.pop().unwrap();
                    tx_env.consumed_gas += gas_cost::POP
                }
                Opcode::MLOAD => {
                    let offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let memory_expansion_cost =
                        current_call_frame.memory.expansion_cost(offset + WORD_SIZE);
                    let gas_cost = gas_cost::MLOAD_STATIC + memory_expansion_cost;
                    if tx_env.consumed_gas + gas_cost > tx_env.gas_limit {
                        break; // should revert the tx
                    }

                    let value = current_call_frame.memory.load(offset);
                    current_call_frame.stack.push(value);
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::MSTORE => {
                    let offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let memory_expansion_cost =
                        current_call_frame.memory.expansion_cost(offset + WORD_SIZE);
                    let gas_cost = gas_cost::MSTORE_STATIC + memory_expansion_cost;
                    if tx_env.consumed_gas + gas_cost > tx_env.gas_limit {
                        break; // should revert the tx
                    }

                    let value = current_call_frame.stack.pop().unwrap();
                    let mut value_bytes = [0u8; WORD_SIZE];
                    value.to_big_endian(&mut value_bytes);

                    current_call_frame.memory.store_bytes(offset, &value_bytes);
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::MSTORE8 => {
                    let offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let memory_expansion_cost =
                        current_call_frame.memory.expansion_cost(offset + 1);
                    let gas_cost = gas_cost::MSTORE8_STATIC + memory_expansion_cost;
                    if tx_env.consumed_gas + gas_cost > tx_env.gas_limit {
                        break; // should revert the tx
                    }

                    let value = current_call_frame.stack.pop().unwrap();
                    let mut value_bytes = [0u8; WORD_SIZE];
                    value.to_big_endian(&mut value_bytes);

                    current_call_frame
                        .memory
                        .store_bytes(offset, value_bytes[WORD_SIZE - 1..WORD_SIZE].as_ref());
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::MSIZE => {
                    if tx_env.consumed_gas + gas_cost::MSIZE > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    current_call_frame
                        .stack
                        .push(current_call_frame.memory.size());
                    tx_env.consumed_gas += gas_cost::MSIZE
                }
                Opcode::GAS => {
                    if tx_env.consumed_gas + gas_cost::GAS > tx_env.gas_limit {
                        break; // should revert the tx
                    }
                    let remaining_gas = tx_env.gas_limit - tx_env.consumed_gas - gas_cost::GAS;
                    current_call_frame.stack.push(remaining_gas.into());
                    tx_env.consumed_gas += gas_cost::GAS
                }
                Opcode::MCOPY => {
                    let dest_offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let src_offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let size = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    if size == 0 {
                        continue;
                    }
                    let words_copied = (size + WORD_SIZE - 1) / WORD_SIZE;
                    let memory_byte_size = ((src_offset + size) as usize).max(dest_offset + size);
                    let memory_expansion_cost =
                        current_call_frame.memory.expansion_cost(memory_byte_size);
                    let gas_cost = gas_cost::MCOPY_STATIC
                        + gas_cost::MCOPY_DYNAMIC_BASE * words_copied as u64
                        + memory_expansion_cost;

                    current_call_frame
                        .memory
                        .copy(src_offset, dest_offset, size);
                    tx_env.consumed_gas += gas_cost
                }
                Opcode::CALL => {
                    let gas = current_call_frame.stack.pop().unwrap();
                    let address =
                        Address::from_low_u64_be(current_call_frame.stack.pop().unwrap().low_u64());
                    let value = current_call_frame.stack.pop().unwrap();
                    let args_offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let args_size = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let ret_offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let ret_size = current_call_frame.stack.pop().unwrap().try_into().unwrap();

                    // check balance
                    if self.balance(&current_call_frame.msg_sender) < value {
                        current_call_frame.stack.push(U256::from(REVERT_FOR_CALL));
                        continue;
                    }

                    // transfer value
                    // transfer(&current_call_frame.msg_sender, &address, value);

                    let callee_bytecode = self.get_account_bytecode(&address);

                    if callee_bytecode.is_empty() {
                        current_call_frame.stack.push(U256::from(SUCCESS_FOR_CALL));
                        continue;
                    }

                    let calldata = current_call_frame
                        .memory
                        .load_range(args_offset, args_size)
                        .into();

                    let new_call_frame = CallFrame {
                        gas,
                        msg_sender: current_call_frame.msg_sender, // caller remains the msg_sender
                        callee: address,
                        bytecode: callee_bytecode,
                        msg_value: value,
                        calldata,
                        ..Default::default()
                    };

                    current_call_frame.return_data_offset = Some(ret_offset);
                    current_call_frame.return_data_size = Some(ret_size);

                    self.call_frames.push(current_call_frame.clone());
                    current_call_frame = new_call_frame;
                }
                Opcode::RETURN => {
                    let offset = current_call_frame.stack.pop().unwrap().try_into().unwrap();
                    let size = current_call_frame.stack.pop().unwrap().try_into().unwrap();

                    let return_data = current_call_frame.memory.load_range(offset, size);

                    if let Some(mut parent_call_frame) = self.call_frames.pop() {
                        if let (Some(ret_offset), Some(_ret_size)) = (
                            parent_call_frame.return_data_offset,
                            parent_call_frame.return_data_size,
                        ) {
                            parent_call_frame
                                .memory
                                .store_bytes(ret_offset, &return_data);
                        }

                        parent_call_frame.stack.push(U256::from(SUCCESS_FOR_RETURN));
                        parent_call_frame.return_data_offset = None;
                        parent_call_frame.return_data_size = None;

                        current_call_frame = parent_call_frame.clone();
                    } else {
                        // excecution completed (?)
                        current_call_frame
                            .stack
                            .push(U256::from(SUCCESS_FOR_RETURN));
                        break;
                    }
                }

                _ => unimplemented!(),
            }
        }
        self.consumed_gas = tx_env.consumed_gas;
        self.call_frames.push(current_call_frame);
    }

    pub fn current_call_frame_mut(&mut self) -> &mut CallFrame {
        self.call_frames.last_mut().unwrap()
    }

    fn get_account_bytecode(&mut self, address: &Address) -> Bytes {
        self.accounts
            .get(address)
            .map_or(Bytes::new(), |acc| acc.bytecode.clone())
    }

    fn balance(&mut self, address: &Address) -> U256 {
        self.accounts
            .get(address)
            .map_or(U256::zero(), |acc| acc.balance)
    }

    pub fn add_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }
}

pub fn arithmetic_shift_right(value: U256, shift: U256) -> U256 {
    let shift_usize: usize = shift.try_into().unwrap(); // we know its not bigger than 256

    if value.bit(255) {
        // if negative fill with 1s
        let shifted = value >> shift_usize;
        let mask = U256::MAX << (256 - shift_usize);
        shifted | mask
    } else {
        value >> shift_usize
    }
}
