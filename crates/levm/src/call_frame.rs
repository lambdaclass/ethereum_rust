use crate::{
    memory::Memory,
    opcodes::Opcode,
    primitives::{Address, Bytes, U256},
};

#[derive(Debug, Clone, Default)]
pub struct CallFrame {
    pub gas: U256,
    pub pc: usize,
    pub msg_sender: Address,
    pub callee: Address,
    pub bytecode: Bytes,
    pub delegate: Option<Address>,
    pub msg_value: U256,
    pub stack: Vec<U256>, // max 1024 in the future
    pub memory: Memory,
    pub calldata: Bytes,
    pub return_data: Bytes,
    // where to store return data of subcall
    pub return_data_offset: Option<usize>,
    pub return_data_size: Option<usize>,
}

impl CallFrame {
    pub fn new(bytecode: Bytes) -> Self {
        Self {
            bytecode,
            ..Default::default()
        }
    }

    pub fn next_opcode(&mut self) -> Option<Opcode> {
        let opcode = self.opcode_at(self.pc);
        self.increment_pc();
        opcode
    }

    pub fn increment_pc_by(&mut self, count: usize) {
        self.pc += count;
    }

    pub fn increment_pc(&mut self) {
        self.increment_pc_by(1);
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn jump(&mut self, jump_address: U256) {
        if !self.valid_jump(jump_address) {
            // Should be a halt when we implement it
            panic!("Invalid jump");
        }
        self.pc = jump_address.as_usize();
    }

    fn valid_jump(&self, jump_address: U256) -> bool {
        self.opcode_at(jump_address.as_usize())
            .map(|opcode| opcode.eq(&Opcode::JUMPDEST))
            .is_some_and(|is_jumpdest| is_jumpdest)
    }

    fn opcode_at(&self, offset: usize) -> Option<Opcode> {
        self.bytecode.get(offset).copied().map(Opcode::from)
    }
}
