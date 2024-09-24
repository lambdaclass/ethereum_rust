use crate::opcodes::Opcode;
use bytes::Bytes;
use ethereum_types::U256;

#[derive(Debug, Clone, Default)]
pub struct VM {
    pub stack: Vec<U256>, // max 1024 in the future
    pc: usize,
}

impl VM {
    pub fn execute(&mut self, mut bytecode: Bytes) {
        loop {
            match self.next_opcode(&mut bytecode).unwrap() {
                Opcode::STOP => break,
                Opcode::ADD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Opcode::MUL => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                }
                Opcode::SUB => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a - b);
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
                Opcode::SDIV => {}
                Opcode::MOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push(a % b);
                }
                Opcode::SMOD => {}
                Opcode::ADDMOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let n = self.stack.pop().unwrap();
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push((a + b) % n);
                }
                Opcode::MULMOD => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let n = self.stack.pop().unwrap();
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                        continue;
                    }

                    self.stack.push((a * b) % n);
                }
                Opcode::EXP => {
                    let base = self.stack.pop().unwrap();
                    let exponent = self.stack.pop().unwrap();
                    self.stack.push(base.pow(exponent));
                }
                Opcode::SIGNEXTEND => {}
                Opcode::PUSH32 => {
                    let next_32_bytes = bytecode.get(self.pc..self.pc + 32).unwrap();
                    let value_to_push = U256::from(next_32_bytes);
                    dbg!(value_to_push);
                    self.stack.push(value_to_push);
                    self.increment_pc_by(32);
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
