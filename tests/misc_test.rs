use cassowary_gb::console::*;
use cassowary_gb::console::cpu::*;
use cassowary_gb::console::memory::*;
use cassowary_gb::console::regids::*;

pub fn load_memory(memory: &mut Memory, data: &[u8]) {
    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }
}


#[test]
pub fn misc() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(3);

    let data = [
        0x00,
        0x10,
        0xE1
    ];

    load_memory(&mut memory, &data);
    cpu.decompile(&memory);
    assert!(false);
}


