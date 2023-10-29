use cassowary_gb::console::cpu::*;
use cassowary_gb::console::cpu::instruction::Instruction::*;
use cassowary_gb::console::memory::*;
use cassowary_gb::console::regids::*;

#[test]
fn misc() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(KBYTE);
    cpu.execute(EI, &mut memory);
    assert!(cpu.is_interruptible());
    cpu.execute(DI, &mut memory);
    assert!(!cpu.is_interruptible());
    cpu.execute(NOP, &mut memory);
    cpu.execute(STOP, &mut memory);
    cpu.execute(HALT, &mut memory);
    assert!(cpu.stop && cpu.halt);
    cpu.execute(CCF, &mut memory);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.execute(CCF, &mut memory);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.execute(SCF, &mut memory);
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
    assert_eq!(cpu.get_reg_int(C), 0x07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(D), 0x07);
    cpu.raw_run(&mut memory);
    assert_eq!(memory.read(0xFF04), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0x00);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0x01);
    cpu.raw_run(&mut memory);
    assert_eq!(memory.read(cpu.get_reg_view(DE)), 0x01);
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
    assert_eq!(cpu.get_reg_view(HL), 0xFF07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(BC), 0xFA10);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0xFF07);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0xFF0C);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0xFF05);
    assert_eq!(memory.read(cpu.get_reg_view(SP)), 0x0C);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0xFF07);
    assert_eq!(memory.read(cpu.get_reg_view(SP)), 0x00);
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
    assert_eq!(cpu.get_reg_int(A), 0b0011_1001);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0b0111_0011);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0b1011_1001);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(A), 0b1101_1100);
    assert_eq!(cpu.get_flag(), 0b0001_0000);

    cpu.raw_run(&mut memory);

    //rlc, rl, rrc, rr
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b11011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory); 
    assert_eq!(cpu.get_reg_int(B), 0b10110100);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b01011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b00101101);
    assert_eq!(cpu.get_flag(), 0b0000_0000);

    //sla, swap, sra, srl
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b01011010);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory); 
    assert_eq!(cpu.get_reg_int(B), 0b10100101);
    assert_eq!(cpu.get_flag(), 0b0000_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b11010010);
    assert_eq!(cpu.get_flag(), 0b0001_0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_int(B), 0b01101001);
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
    assert_eq!(cpu.get_reg_view(BC), 0x0813);
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
    assert_eq!(cpu.get_reg_view(BC), 0x0813); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0x7FFF); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0x0001);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(DE), 0xFFFF);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0x0814);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00000000); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0x0813);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00110000); 
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0x7FF9);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0x8009);
    assert_eq!(cpu.get_flag() & 0xF0, 0b00010000); 
    cpu.raw_run(&mut memory);
}

#[test]
fn control_flow() {
    let mut cpu = SharpSM83::new();
    let mut memory = Memory::new(8*KBYTE);
    let instructions = vec![
        vec![0x31, 0xFF, 0x7F], // ld SP nn
        vec![0x21, 0x0B, 0x00], // ld hl nn
        vec![0x37], // SCF
        vec![0xC3, 0x00, 0x01], // jp nn
        vec![0xE9], // jp hl
        vec![0x18, 0x03], // jr PC + dd
        vec![0x00],
        vec![0x00],
        vec![0x00],
        vec![0x30, 0x00], // jr cc, PC + dd
        vec![0xCD, 0x04, 0x01], // call nn
        vec![0xD4, 0x05, 0x01], // call cc, nn
        vec![0x00], // rst n
        vec![0x00],
    ];
    
    memory.write(0x0100, 0x3F); // ccf flip carry
    memory.write(0x0101, 0xD2); // jp nc nn
    memory.write(0x0102, 0x0A);
    memory.write(0x0103, 0x00);
    memory.write(0x0104, 0xC9); // ret
    memory.write(0x0105, 0x37); // scf
    memory.write(0x0106, 0xD8); // ret c

    let data: Vec<u8> = instructions.clone().into_iter().flatten().collect();

    for i in 0..data.len() {
        let byte = data[i];
        memory.write(i as u16, byte);
    }

    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(SP), 0x7FFF);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_reg_view(HL), 0x000B);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_flag() & 0x10, 0x10);
    
    let pc_ret = cpu.pc;
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 0x0100);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_flag() & 0x10, 0x00);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 0x000A);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, cpu.get_reg_view(HL));
    let pc_before = cpu.pc + 2;
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, pc_before + 0x0003);
    let pc_before = cpu.pc + 2;
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, pc_before + 0x0000);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 0x0104);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 21);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 0x0105);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.get_flag() & 0x10, 0x10);
    cpu.raw_run(&mut memory);
    assert_eq!(cpu.pc, 24);
    cpu.raw_run(&mut memory);
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
    assert_eq!(cpu.get_reg_view(BC), 0x0813);
}


