use cassowary_gb::bytes::u8_to_u16;
use cassowary_gb::console::*;
use cassowary_gb::console::cpu::*;
use cassowary_gb::console::cpu::instruction::Instruction::*;
use cassowary_gb::console::memory::*;
use cassowary_gb::console::regids::*;

#[test]
fn misc() {
    let mut cpu = SharpSM83::new();
    let memory = Memory::new(KBYTE);
    cpu.execute(EI, &memory);
    assert!(cpu.is_interruptible());
    cpu.execute(DI, &memory);
    assert!(!cpu.is_interruptible());
    cpu.execute(NOP, &memory);
    cpu.execute(STOP, &memory);
    cpu.execute(HALT, &memory);
    assert!(cpu.stop && cpu.halt);
    cpu.execute(CCF, &memory);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.execute(CCF, &memory);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.execute(SCF, &memory);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
}

#[test]
fn load_8bit() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x0E, 0x07], // ld c, n
        vec![0x3E, 0x01], // ld a, n
        vec![0x51], // ld d, c
        vec![0xE0, 0x04], // ldh $FF00 + (n), a
        vec![0xF2], // ldh a, $FF00 + (c)
        vec![0xFA, 0x04, 0xFF], // LD A (nn)
        vec![0x12], // LD (DE) A
        vec![0x00],
    ];
    
    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();
    
    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(C), 0x07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(D), 0x07);
    cpu.raw_run(&mut memory);
    assert_eq!(memory.read(0xFF04), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0x00);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(memory.read(cpu.get_rr(DE)), 0x01);
}

#[test]
fn load_16bit() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x21, 0x07, 0xFF], // ld HL nn
        vec![0x01, 0x10, 0xFA], // ld BC nn
        vec![0xF9], // ld sp hl
        vec![0xF8, 0x05], // ld hl, sp + e
        vec![0xE5], // push hl
        vec![0xC1], // pop hl
        vec![0x08, 0x00, 0xFF], // ld (nn) sp
        vec![0x00],
    ];
    
    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();
    
    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0xFF07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(BC), 0xFA10);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0xFF07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0xFF0C);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0xFF05);
    assert_eq!(memory.read(cpu.get_rr(SP)), 0x0C);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0xFF07);
    assert_eq!(memory.read(cpu.get_rr(SP)), 0x00);
    cpu.raw_run(&mut memory);
    assert_eq!(memory.read(0xFF00), 0x07);
    assert_eq!(memory.read(0xFF01), 0xFF);
    cpu.raw_run(&mut memory);
}

#[test]
fn rotates_and_shifts() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);

    let instructions = vec![
        vec![0x21, 0x00, 0xFF], // ld HL nn
        vec![0x3E, 0b1001_1100], // ld A, n
        vec![0x77], // ld (HL), A
        vec![0x07], // RLCA
        vec![0x17], // RLA
        vec![0x0F], // RRCA
        vec![0x1F], // RRA
        vec![0x06, 0b01101101], // ld b, n
        vec![0xCB, 0x00], // rlc r -> 0b1101101x, c=
        vec![0xCB, 0x10], // rl r
        vec![0xCB, 0x08], // rrc r
        vec![0xCB, 0x18], // rr r
        vec![0xCB, 0x20], // sla r
        vec![0xCB, 0x30], // swap r
        vec![0xCB, 0x28], // sra r
        vec![0xCB, 0x38], // srl r
        vec![0x00],
    ];

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    cpu.raw_run(&mut memory);
    cpu.raw_run(&mut memory);

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0b0011_1001);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0b0111_0011);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0b1011_1001);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(A), 0b1101_1100);
    assert_eq!(cpu.get_flag(), 0b0001_0000);

    cpu.raw_run(&mut memory);

    //rlc, rl, rrc, rr
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b11011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory); 
    assert_eq!(cpu.get_reg(B), 0b10110100);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b01011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b00101101);
    assert_eq!(cpu.get_flag(), 0b0000_0000);

    //sla, swap, sra, srl
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b01011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory); 
    assert_eq!(cpu.get_reg(B), 0b10100101);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b11010010);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg(B), 0b01101001);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory);
}

#[test]
fn arithmetic_8bit() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x01, 0x13, 0x08], // ld BC nn
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
    ];

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(BC), 0x0813);
}

#[test]
fn arithmetic_16bit(){
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x01, 0x13, 0x08], // ld BC nn
        vec![0x31, 0xFF, 0x7F], // ld SP nn
        vec![0x23], // inc HL
        vec![0x1B], // dec DE
        vec![0x09], // add HL BC
        vec![0x19], // add HL DE
        vec![0xE8, 0xFA], // add sp, e=$-5
        vec![0xF8, 0x10], // ld hl, e=$-13
        vec![0x00],
    ];

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(BC), 0x0813); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0x7FFF); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0x0001);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(DE), 0xFFFF);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0x0814);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00000000); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0x0813);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00110000); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0x7FF9);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(HL), 0x8009);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00010000); 
    cpu.raw_run(&mut memory);
}

#[test]
fn control_flow() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x31, 0xFF, 0x7F], // ld SP nn
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
    ];

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(SP), 0x7FFF);
}

#[test]
fn prefix() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x01, 0x13, 0x08], // ld BC nn
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x00],
    ];

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_rr(BC), 0x0813);
}


