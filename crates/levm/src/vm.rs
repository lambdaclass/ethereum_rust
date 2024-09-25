use crate::opcodes::Opcode;
use bytes::Bytes;
use ethereum_types::U256;

#[derive(Debug, Clone, Default)]
pub struct VM {
    pub stack: Vec<U256>, // max 1024 in the future
    pub memory: Memory,
    pub pc: usize,
}

/// Shifts the value to the right by 255 bits and checks the most significant bit is a 1
fn is_negative(value: U256) -> bool {
    value >> 255 == U256::one()
}
/// Converts a positive value to a negative one in two's complement
fn to_negative(value: U256) -> U256 {
    !value + U256::one()
}

impl VM {
    pub fn execute(&mut self, mut bytecode: Bytes) {
        loop {
            match self.next_opcode(&mut bytecode).unwrap() {
                Opcode::STOP => break,
                Opcode::ADD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a.overflowing_add(b).0);
                }
                Opcode::MUL => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a.overflowing_mul(b).0);
                }
                Opcode::SUB => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a.overflowing_sub(b).0);
                }
                Opcode::DIV => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push(a / b);
                }
                Opcode::SDIV => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    let a_is_negative = is_negative(a);
                    let b_is_negative = is_negative(b);
                    let a = if a_is_negative { to_negative(a) } else { a };
                    let b = if b_is_negative { to_negative(b) } else { b };
                    let result = a / b;
                    let result_is_negative = a_is_negative ^ b_is_negative;
                    let result = if result_is_negative {
                        to_negative(result)
                    } else {
                        result
                    };

                    self.stack.push(result);
                }
                Opcode::MOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push(a % b);
                }
                Opcode::SMOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    let a_is_negative = is_negative(a);
                    let b_is_negative = is_negative(b);
                    let a = if a_is_negative { to_negative(a) } else { a };
                    let b = if b_is_negative { to_negative(b) } else { b };
                    let result = a % b;
                    let result_is_negative = a_is_negative ^ b_is_negative;
                    let result = if result_is_negative {
                        to_negative(result)
                    } else {
                        result
                    };

                    self.stack.push(result);
                }
                Opcode::ADDMOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let n = self.stack.pop().unwrap();
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push((a.overflowing_add(b).0) % n);
                }
                Opcode::MULMOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let n = self.stack.pop().unwrap();
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push((a.overflowing_mul(b).0) % n);
                }
                Opcode::EXP => {
                    let base = self.stack.pop().unwrap();
                    let exponent = self.stack.pop().unwrap();
                    self.stack.push(base.overflowing_pow(exponent).0);
                }
                Opcode::SIGNEXTEND => {
                    let byte_size = self.stack.pop().unwrap();
                    let value_to_extend = self.stack.pop().unwrap();

                    let bits_per_byte = U256::from(8);
                    let sign_bit_position_on_byte = 7;
                    let max_byte_size = 31;

                    let byte_size = byte_size.min(U256::from(max_byte_size));
                    let sign_bit_index = bits_per_byte * byte_size + sign_bit_position_on_byte;
                    let is_negative = value_to_extend.bit(sign_bit_index.as_usize());
                    let sign_bit_mask = (U256::one() << sign_bit_index) - U256::one();
                    let result = if is_negative {
                        value_to_extend.saturating_add(!sign_bit_mask)
                    } else {
                        value_to_extend & sign_bit_mask
                    };
                    self.stack.push(result);
                }
                Opcode::PUSH32 => {
                    let next_32_bytes = bytecode.get(self.pc..self.pc + 32).unwrap();
                    let value_to_push = U256::from(next_32_bytes);
                    dbg!(value_to_push);
                    self.stack.push(value_to_push);
                    self.increment_pc_by(32);
                }
                Opcode::MLOAD => {
                    // spend_gas(3);
                    let offset = self.stack.pop().unwrap().try_into().unwrap();
                    let value = self.memory.load(offset);
                    self.stack.push(value);
                }
                Opcode::MSTORE => {
                    // spend_gas(3);
                    let offset = self.stack.pop().unwrap().try_into().unwrap();
                    let value = self.stack.pop().unwrap();
                    let mut value_bytes = [0u8; 32];
                    value.to_big_endian(&mut value_bytes);

                    self.memory.store_bytes(offset, &value_bytes);
                }
                Opcode::MSTORE8 => {
                    // spend_gas(3);
                    let offset = self.stack.pop().unwrap().try_into().unwrap();
                    let value = self.stack.pop().unwrap();
                    let mut value_bytes = [0u8; 32];
                    value.to_big_endian(&mut value_bytes);

                    self.memory
                        .store_bytes(offset, value_bytes[31..32].as_ref());
                }
                Opcode::MSIZE => {
                    // spend_gas(2);
                    self.stack.push(self.memory.size());
                }
                Opcode::MCOPY => {
                    // spend_gas(3) + dynamic gas
                    let dest_offset = self.stack.pop().unwrap().try_into().unwrap();
                    let src_offset = self.stack.pop().unwrap().try_into().unwrap();
                    let size = self.stack.pop().unwrap().try_into().unwrap();
                    if size == 0 {
                        continue;
                    }

                    self.memory.copy(src_offset, dest_offset, size);
                }
            }
        }
    }

    fn next_opcode(&mut self, opcodes: &mut Bytes) -> Option<Opcode> {
        let opcode = opcodes.get(self.pc).copied().map(Opcode::from);
        self.increment_pc();
        opcode
    }

    fn increment_pc_by(&mut self, count: usize) {
        self.pc += count;
    }

    fn increment_pc(&mut self) {
        self.increment_pc_by(1);
    }

    pub fn pc(&self) -> usize {
        self.pc
    }
}

#[derive(Debug, Clone, Default)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn resize(&mut self, offset: usize) {
        if offset.next_multiple_of(32) > self.data.len() {
            self.data.resize(offset.next_multiple_of(32), 0);
        }
    }

    pub fn load(&mut self, offset: usize) -> U256 {
        self.resize(offset + 32);
        let value_bytes: [u8; 32] = self
            .data
            .get(offset..offset + 32)
            .unwrap()
            .try_into()
            .unwrap();
        U256::from(value_bytes)
    }

    pub fn store_bytes(&mut self, offset: usize, value: &[u8]) {
        let len = value.len();
        self.resize(offset + len);
        self.data
            .splice(offset..offset + len, value.iter().copied());
    }

    pub fn size(&self) -> U256 {
        U256::from(self.data.len())
    }

    pub fn copy(&mut self, src_offset: usize, dest_offset: usize, size: usize) {
        let max_size = std::cmp::max(src_offset + size, dest_offset + size);
        self.resize(max_size);
        let mut temp = vec![0u8; size];

        temp.copy_from_slice(&self.data[src_offset..src_offset + size]);

        self.data[dest_offset..dest_offset + size].copy_from_slice(&temp);
    }
}
