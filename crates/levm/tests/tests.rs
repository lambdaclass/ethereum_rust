use bytes::Bytes;
use ethereum_types::U256;
use levm::{operations::Operation, vm::VM};

// cargo test -p 'levm'

fn new_vm_with_ops(operations: &[Operation]) -> VM {
    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    VM::new(bytecode)
}

#[test]
fn test() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::one()),
        Operation::Push32(U256::zero()),
        Operation::Add,
        Operation::Stop,
    ]);

    vm.execute();

    assert!(vm.current_call_frame().stack.pop().unwrap() == U256::one());
    assert!(vm.current_call_frame().pc() == 68);

    println!("{vm:?}");
}

#[test]
fn and_basic() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::from(0b1100)),
        Operation::And,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1000));
}

#[test]
fn and_binary_with_zero() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::zero()),
        Operation::And,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::zero());
}

#[test]
fn and_with_hex_numbers() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xFFFF)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::And,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xF0F0));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xF000)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::And,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xF000));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xB020)),
        Operation::Push32(U256::from(0x1F0F)),
        Operation::And,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1000000000000));
}

#[test]
fn or_basic() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::from(0b1100)),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1110));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::zero()),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1010));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(u64::MAX)),
        Operation::Push32(U256::zero()),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xFFFFFFFFFFFFFFFF as u64));
}

#[test]
fn or_with_hex_numbers() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xFFFF)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xFFFF));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xF000)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xF0F0));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xB020)),
        Operation::Push32(U256::from(0x1F0F)),
        Operation::Or,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1011111100101111));
}

#[test]
fn xor_basic() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::from(0b1100)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b110));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0b1010)),
        Operation::Push32(U256::zero()),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b1010));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(u64::MAX)),
        Operation::Push32(U256::zero()),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(u64::MAX));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(u64::MAX)),
        Operation::Push32(U256::from(u64::MAX)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::zero());
}

#[test]
fn xor_with_hex_numbers() {
    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xF0)),
        Operation::Push32(U256::from(0xF)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xFF));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xFF)),
        Operation::Push32(U256::from(0xFF)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::zero());

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xFFFF)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xF0F));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0xF000)),
        Operation::Push32(U256::from(0xF0F0)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0xF0));

    let mut vm = new_vm_with_ops(&[
        Operation::Push32(U256::from(0x4C0F)),
        Operation::Push32(U256::from(0x3A4B)),
        Operation::Xor,
        Operation::Stop,
    ]);

    vm.execute();

    let result = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(result, U256::from(0b111011001000100));
}

#[test]
fn mstore() {
    let operations = [
        Operation::Push32(U256::from(0x33333)),
        Operation::Push32(U256::zero()),
        Operation::Mstore,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let stored_value = vm.current_call_frame().memory.load(0);

    assert_eq!(stored_value, U256::from(0x33333));

    let memory_size = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(memory_size, U256::from(32));
}

#[test]
fn mstore8() {
    let operations = [
        Operation::Push32(U256::from(0xAB)), // value
        Operation::Push32(U256::zero()),     // offset
        Operation::Mstore8,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let stored_value = vm.current_call_frame().memory.load(0);

    let mut value_bytes = [0u8; 32];
    stored_value.to_big_endian(&mut value_bytes);

    assert_eq!(value_bytes[0..1], [0xAB]);
}

#[test]
fn mcopy() {
    let operations = [
        Operation::Push32(U256::from(32)),      // size
        Operation::Push32(U256::from(0)),       // source offset
        Operation::Push32(U256::from(64)),      // destination offset
        Operation::Push32(U256::from(0x33333)), // value
        Operation::Push32(U256::from(0)),       // offset
        Operation::Mstore,
        Operation::Mcopy,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let copied_value = vm.current_call_frame().memory.load(64);
    assert_eq!(copied_value, U256::from(0x33333));

    let memory_size = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(memory_size, U256::from(96));
}

#[test]
fn mload() {
    let operations = [
        Operation::Push32(U256::from(0x33333)), // value
        Operation::Push32(U256::zero()),        // offset
        Operation::Mstore,
        Operation::Push32(U256::zero()), // offset
        Operation::Mload,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let loaded_value = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(loaded_value, U256::from(0x33333));
}

#[test]
fn msize() {
    let operations = [Operation::Msize, Operation::Stop];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let initial_size = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(initial_size, U256::from(0));

    let operations = [
        Operation::Push32(U256::from(0x33333)), // value
        Operation::Push32(U256::zero()),        // offset
        Operation::Mstore,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    vm = VM::new(bytecode);

    vm.execute();

    let after_store_size = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(after_store_size, U256::from(32));

    let operations = [
        Operation::Push32(U256::from(0x55555)), // value
        Operation::Push32(U256::from(64)),      // offset
        Operation::Mstore,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    vm = VM::new(bytecode);

    vm.execute();

    let final_size = vm.current_call_frame().stack.pop().unwrap();
    assert_eq!(final_size, U256::from(96));
}

#[test]
fn mstore_mload_offset_not_multiple_of_32() {
    let operations = [
        Operation::Push32(0xabcdef.into()), // value
        Operation::Push32(10.into()),       // offset
        Operation::Mstore,
        Operation::Push32(10.into()), // offset
        Operation::Mload,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let memory_size = vm.current_call_frame().stack.pop().unwrap();
    let loaded_value = vm.current_call_frame().stack.pop().unwrap();

    assert_eq!(loaded_value, U256::from(0xabcdef));
    assert_eq!(memory_size, U256::from(64));

    //check with big offset

    let operations = [
        Operation::Push32(0x123456.into()), // value
        Operation::Push32(2000.into()),     // offset
        Operation::Mstore,
        Operation::Push32(2000.into()), // offset
        Operation::Mload,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    vm = VM::new(bytecode);

    vm.execute();

    let memory_size = vm.current_call_frame().stack.pop().unwrap();
    let loaded_value = vm.current_call_frame().stack.pop().unwrap();

    assert_eq!(loaded_value, U256::from(0x123456));
    assert_eq!(memory_size, U256::from(2048));
}

#[test]
fn mload_uninitialized_memory() {
    let operations = [
        Operation::Push32(50.into()), // offset
        Operation::Mload,
        Operation::Msize,
        Operation::Stop,
    ];

    let bytecode = operations
        .iter()
        .flat_map(Operation::to_bytecode)
        .collect::<Bytes>();

    let mut vm = VM::new(bytecode);

    vm.execute();

    let memory_size = vm.current_call_frame().stack.pop().unwrap();
    let loaded_value = vm.current_call_frame().stack.pop().unwrap();

    assert_eq!(loaded_value, U256::zero());
    assert_eq!(memory_size, U256::from(96));
}
