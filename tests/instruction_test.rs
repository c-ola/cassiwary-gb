use cassowary_gb::console::cpu::instruction::{Instruction::*, *};

#[test]
fn decode_block0() {
    let mut opcode = 0b00000000;
    // nop
    let res = Instruction::decode(opcode);
    assert_eq!(res, NOP);

    // ld r16 imm16
    opcode = 0b00_00_0001;
    for i in 0..4 {
        let res = Instruction::decode(opcode);
        assert_eq!(res, LDrrnn(i));
        opcode = opcode + 0b00_01_0000
    }

    // ld [r16mem], a
    opcode = 0b00_00_0010;
    for i in 0..4 {
        let res = Instruction::decode(opcode);
        assert_eq!(res, LDRRawA(i));
        opcode = opcode + 0b00_01_0000
    }

    // ld a, [r16mem]
    opcode = 0b00_00_1010;
    for i in 0..4 {
        let res = Instruction::decode(opcode);
        assert_eq!(res, LDAwRRa(i));
        opcode = opcode + 0b00_01_0000
    }

    // ld [imm16], sp
    opcode = 0b00_00_1000;
    let res = Instruction::decode(opcode);
    assert_eq!(res, LDNNawSP);

    // inc r16, dec 16, add hl, r16
    for i in 0..4 {
        let operand = i << 4;
        let res = Instruction::decode(0b00_00_0011 + operand);
        assert_eq!(res, INCrr(i));
        let res = Instruction::decode(0b00_00_1011 + operand);
        assert_eq!(res, DECrr(i));
        let res = Instruction::decode(0b00_00_1001 + operand);
        assert_eq!(res, ADDHLrr(i));
    }
    
    // inc r8, dec r8
    // ld r8, imm8
    for i in 0..8 {
        let operand = i << 3;
        let res = Instruction::decode(0b00_000_100 + operand);
        assert_eq!(res, IncR(i));
        let res = Instruction::decode(0b00_000_101 + operand);
        assert_eq!(res, DecR(i));
        let res = Instruction::decode(0b00_000_110 + operand);
        assert_eq!(res, LDRwN(i));
    }

    // rlca, rrca, rla, rra, daa, cpl, scf, ccf
    let instrs = vec![
        (0b00000_111, RLCA),
        (0b00001_111, RRCA),
        (0b00010_111, RLA),
        (0b00011_111, RRA),
        (0b00100_111, DAA),
        (0b00101_111, CPL),
        (0b00110_111, SCF),
        (0b00111_111, CCF),
    ];
    for instr in instrs {
        assert_eq!(Instruction::decode(instr.0), instr.1); 
    }

    // jr imm8, jr cond imm8
    assert_eq!(Instruction::decode(0b00011000), JRe);
    for i in 0..4 {
        assert_eq!(Instruction::decode(0b001_00_000 + (i << 3)), JRcce(i)); 
    }

    assert_eq!(Instruction::decode(0b00010000), STOP);
}

#[test]
fn decode_block1() {
    //ld r8, r8
    let prefix = 0b01_000_000;
    for dest in 0..8 {
        for src in 0..8 {
            let opcode = prefix + (dest << 3) + src;
            if opcode == 0b01110110 { assert_eq!(Instruction::decode(opcode), HALT); }
            else { assert_eq!(Instruction::decode(opcode), LDRwR(dest, src)); }
        }
    }
}

#[test]
fn decode_block2() {
    for operand in 0..8 {
        assert_eq!(Instruction::decode(0b10000_000 + operand), Add(operand));
        assert_eq!(Instruction::decode(0b10001_000 + operand), Adc(operand));
        assert_eq!(Instruction::decode(0b10010_000 + operand), Sub(operand));
        assert_eq!(Instruction::decode(0b10011_000 + operand), Sbc(operand));
        assert_eq!(Instruction::decode(0b10100_000 + operand), And(operand));
        assert_eq!(Instruction::decode(0b10101_000 + operand), Xor(operand));
        assert_eq!(Instruction::decode(0b10110_000 + operand), Or(operand));
        assert_eq!(Instruction::decode(0b10111_000 + operand), Cmp(operand));
    }
}

#[test]
fn decode_block3() {
    assert_eq!(Instruction::decode(0b11000_110), Addn);
    assert_eq!(Instruction::decode(0b11001_110), Adcn);
    assert_eq!(Instruction::decode(0b11010_110), Subn);
    assert_eq!(Instruction::decode(0b11011_110), Sbcn);
    assert_eq!(Instruction::decode(0b11100_110), Andn);
    assert_eq!(Instruction::decode(0b11101_110), Xorn);
    assert_eq!(Instruction::decode(0b11110_110), Orn);
    assert_eq!(Instruction::decode(0b11111_110), Cmpn);
    
    // ret cc, jp cc imm16, call cc imm16
    for i in 0..4 {
        let cond = i << 3;
        assert_eq!(Instruction::decode(0b110_00_000 + cond), RETcc(i));
        assert_eq!(Instruction::decode(0b110_00_010 + cond), JPccnn(i));
        assert_eq!(Instruction::decode(0b110_00_100 + cond), CALLccnn(i));
    }
    assert_eq!(Instruction::decode(0b110_01_001), RET);
    assert_eq!(Instruction::decode(0b110_11_001), RETI);
    assert_eq!(Instruction::decode(0b110_00_011), JPnn);
    assert_eq!(Instruction::decode(0b111_01_001), JPHL);
    assert_eq!(Instruction::decode(0b110_01_101), CALLnn);

    // rst tgt3
    for i in 0..8 {
        assert_eq!(Instruction::decode(0b11_000_111 + (i << 3)), RSTn(i));
    }
    
    // pop r16, push r16
    for i in 0..4 {
        assert_eq!(Instruction::decode(0b11_00_0001 + (i << 4)), POPrr(i));
        assert_eq!(Instruction::decode(0b11_00_0101 + (i << 4)), PUSHrr(i));
    }

    // check CB in own function
    assert_eq!(Instruction::decode(0xCB), CB);

    // LDH // FIX THIS DAMN SYNTAX FOR LDH
    assert_eq!(Instruction::decode(0b111_000_10), LDH(true, false));
    assert_eq!(Instruction::decode(0b111_000_00), LDH(true, true));
    assert_eq!(Instruction::decode(0b111_100_10), LDH(false, false));
    assert_eq!(Instruction::decode(0b111_100_00), LDH(false, true));
    
    // other loads with imm16 // fix this garbage as well please
    assert_eq!(Instruction::decode(0b111_010_10), LDAwNNa(true));
    assert_eq!(Instruction::decode(0b111_110_10), LDAwNNa(false));
    
    // random stack pointer stuff
    assert_eq!(Instruction::decode(0b11101000), AddSpE);
    assert_eq!(Instruction::decode(0b11111000), LDHLwSP);
    assert_eq!(Instruction::decode(0b11111001), LDSPwHL);
    
    // interupts
    assert_eq!(Instruction::decode(0b11110011), DI);
    assert_eq!(Instruction::decode(0b11111011), EI);
}

#[test]
fn decode_cb() {
    for operand in 0..8 {
        assert_eq!(Instruction::decode_cb(0b00000_000 + operand), RLCr(operand));
        assert_eq!(Instruction::decode_cb(0b00001_000 + operand), RRCr(operand));
        assert_eq!(Instruction::decode_cb(0b00010_000 + operand), RLr(operand));
        assert_eq!(Instruction::decode_cb(0b00011_000 + operand), RRr(operand));
        assert_eq!(Instruction::decode_cb(0b00100_000 + operand), SLAr(operand));
        assert_eq!(Instruction::decode_cb(0b00101_000 + operand), SRAr(operand));
        assert_eq!(Instruction::decode_cb(0b00110_000 + operand), SWAPr(operand));
        assert_eq!(Instruction::decode_cb(0b00111_000 + operand), SRLr(operand));
    }
    for n in 0..8 {
        for r in 0..8 {
            assert_eq!(Instruction::decode_cb(0b01_000_000 + (n << 3) + r), BITnr{n, r});
            assert_eq!(Instruction::decode_cb(0b10_000_000 + (n << 3) + r), RESnr{n, r});
            assert_eq!(Instruction::decode_cb(0b11_000_000 + (n << 3) + r), SETnr{n, r});
        }
    }
}

