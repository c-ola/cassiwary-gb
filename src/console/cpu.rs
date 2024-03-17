pub mod identifiers;
pub mod instruction;

use crate::bytes::*;
use crate::console::regids::*;
use crate::cpu::identifiers::*;
use crate::cpu::instruction::*;
use crate::memory::*;

use Instruction::*;

use std::fmt;

// the cpu
pub struct SharpSM83 {
    // 8-bit general purpose
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    //8-bit flag / 0-3 grounded to 0, 4 carry flag C, 5, half-carry H, 6 negative N, 7 zero Z
    f: u8,
    ime: u8,

    //16-bit special purpose
    pub pc: u16,
    sp: u16,

    pub stop: bool,
    pub halt: bool,

    instructions_executed: usize,
}

impl SharpSM83 {
    pub fn is_interruptible(&self) -> bool {
        self.ime == 1
    }
    pub fn new() -> SharpSM83 {
        SharpSM83 {
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            f: 0x00, // Z N H C 0 0 0 0

            ime: 0,

            pc: 0x0000,
            sp: 0x0000,

            stop: false,
            halt: false,

            instructions_executed: 0,
        }
    }

    //returns the number of clock cycles for the instruction
    pub fn run(&mut self, memory: &mut Memory) -> Option<usize> {
        if !self.stop {
            let handled_interrupt = if self.ime == 1 {
                self.handle_interrupt(memory)
            } else {
                false
            };
            if handled_interrupt {
            } else if !self.halt {
                let opcode = self.fetch(memory);
                let instr = Instruction::decode(opcode);
                self.execute(instr, memory);
                self.instructions_executed += 1;
            }
            return Some(4);
        }
        Some(0)
    }
    

    pub fn get_instr_executed(&self) -> usize {
        self.instructions_executed
    }

    pub fn raw_run(&mut self, memory: &mut Memory) {
        let opcode = self.fetch(memory);
        let instr = Instruction::decode(opcode);
        //eprintln!("{:?}", instr);
        self.execute(instr, memory);
        //self.print_info();
    }

    fn fetch(&mut self, memory: &Memory) -> u8 {
        // ---- get instruction from memory  ----
        let opcode = memory.read(self.pc);

        //eprintln!("fetched {0:#04X} at pc: {1:#04X}", opcode, self.pc);

        self.pc = self.pc.overflowing_add(1).0;

        opcode
    }

    pub fn reset(&mut self) {
        self.stop = false;
    }

    /// Executes the specified instruction on the memory
    pub fn execute(&mut self, instr: Instruction, memory: &mut Memory) {
        if instr != NOP && (self.pc < 700 || self.pc > 754) {
            //eprintln!("{}, {instr:?}", self.pc);
        }
        match instr {
            ErrInstr { opcode } => match opcode {
                0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                    panic!("Instruction Undefined {opcode:#02X}")
                }
                _ => panic!("Invalid Instruction {opcode:#02X}"),
            },
            NOP => (),
            STOP => self.stop = true,
            HALT => self.halt = true,
            DI => self.ime = 0,
            EI => self.ime = 1,
            LDRwR(r1, r2) => self.set_reg(r1, self.get_reg(r2, memory), memory),
            LDRwN(r) => {
                let n = self.fetch(memory);
                self.set_reg(r, n, memory);
            }
            LDH(from, n) => {
                let loc = u8_to_u16(
                    0xFF,
                    match n {
                        true => self.fetch(memory),
                        false => self.get_reg_int(C),
                    },
                );
                match from {
                    true => memory.write(loc, self.get_reg_int(A)),
                    false => self.set_reg_int(A, memory.read(loc)),
                }
            }
            LDAwNNa(from) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let loc = u8_to_u16(msb, lsb);
                match from {
                    false => self.set_reg(A, memory.read(loc), memory),
                    true => self.write(loc, self.get_reg_int(A), memory),
                }
            }
            LDRRawA(rr) => {
                let loc = self.get_reg_view_addr(rr);
                self.write(loc, self.get_reg_int(A), memory);
            }
            LDAwRRa(rr) => {
                let loc = self.get_reg_view_addr(rr);
                self.set_reg(A, memory.read(loc), memory);
            }
            LDrrnn(rr) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);
                self.set_rr(rr, nn);
            }
            LDNNawSP => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let loc = u8_to_u16(msb, lsb);
                self.write(loc, low_u16(self.sp), memory);
                self.write(loc + 1, high_u16(self.sp), memory);
            }
            PUSHrr(rr) => {
                let rr = self.get_reg_view_int(rr);
                let msb = high_u16(rr);
                let lsb = low_u16(rr);

                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, msb, memory);
                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, lsb, memory);
            }
            POPrr(rr) => {
                let lsb = memory.read(self.sp);
                self.sp += 1;
                let msb = memory.read(self.sp);
                self.sp += 1;
                self.set_reg_view_int(rr, u8_to_u16(msb, lsb));
            }
            LDHLwSP => {
                let e = self.fetch(memory);
                let result = i16_add(self.get_reg_view(SP) as i16, e as i8 as i16);
                self.set_rr(HL, result.0 as u16);
                self.set_flags(false, false, result.2, result.1);
            }
            LDSPwHL => {
                self.set_rr(SP, self.get_reg_view(HL));
            }
            DecR(r) => {
                let rv = self.get_reg(r, memory);
                let result = rv.overflowing_sub(1);
                self.set_reg(r, result.0, memory);

                let half_c = (rv & 0xF).overflowing_sub(1).1;
                self.set_flag(FLAG_Z, result.0 == 0);
                self.set_flag(FLAG_N, true);
                self.set_flag(FLAG_H, half_c);
            }
            IncR(r) => {
                let rv = self.get_reg(r, memory);
                let result = rv.overflowing_add(1);
                self.set_reg(r, result.0, memory);

                let half_c = (rv & 0xF) + (0x1) > 0xF;
                self.set_flag(FLAG_Z, result.0 == 0);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, half_c);
            }
            Add(_) | Sub(_) | And(_) | Or(_) | Adc(_) | Sbc(_) | Xor(_) | Cmp(_) | Addn | Subn
            | Andn | Orn | Adcn | Sbcn | Xorn | Cmpn => {
                let n = match instr {
                    Addn | Subn | Andn | Orn | Adcn | Sbcn | Xorn | Cmpn => self.fetch(memory),
                    Add(r) | Sub(r) | And(r) | Or(r) | Adc(r) | Sbc(r) | Xor(r) | Cmp(r) => {
                        self.get_reg(r, memory)
                    }
                    _ => panic!("not supposed to get here"),
                };
                let result = match instr {
                    //kinda yuck
                    Add(_) | Addn => u8_add(self.a, n),
                    Sub(_) | Subn => u8_sub(self.a, n),
                    And(_) | Andn => u8_and(self.a, n),
                    Or(_) | Orn => u8_or(self.a, n),
                    Adc(_) | Adcn => u8_addc(self.a, n, self.get_flag_bit(FLAG_C)),
                    Sbc(_) | Sbcn => u8_subc(self.a, n, self.get_flag_bit(FLAG_C)),
                    Xor(_) | Xorn => u8_xor(self.a, n),
                    Cmp(_) | Cmpn => u8_cmp(self.a, n),

                    _ => (0, 0),
                };
                match instr {
                    Cmp(_) | Cmpn => (),
                    _ => self.set_reg(A, result.0, memory),
                }
                self.f = result.1;
            }
            AddSpE => {
                let e = self.fetch(memory);
                let result = i16_add(self.sp as i16, e as i8 as i16);
                self.sp = result.0 as u16;
                self.set_flags(false, false, result.2, result.1);
                //self.set_carry_flags(result.1, result.2);
            }
            INCrr(rr) | DECrr(rr) | ADDHLrr(rr) => {
                let rrv = self.get_reg_view(rr);
                let mut rr_key = rr;
                let result = match instr {
                    INCrr(_) => u16_add(rrv, 1),
                    DECrr(_) => u16_sub(rrv, 1),
                    ADDHLrr(_) => {
                        let hl = self.get_reg_view(HL);
                        rr_key = HL;
                        let result = u16_add(hl, rrv);
                        self.set_flag(FLAG_N, false);
                        self.set_flag(FLAG_H, result.2);
                        self.set_flag(FLAG_C, result.1);
                        result
                    }
                    _ => panic!("Should not be here"),
                };

                self.set_rr(rr_key, result.0);
            }

            DAA => {
                let mut correction = 0;
                let mut value = self.a;

                let mut set_flag_c = 0u8;

                let n_flag = self.f & 0b0100_0000 != 0;
                let h_flag = self.f & 0b0010_0000 != 0;
                let c_flag = self.f & 0b0001_0000 != 0;

                if h_flag || (!n_flag && (value & 0xf) > 0x9) {
                    correction |= 0x6;
                }

                if c_flag || (!n_flag && value > 0x99) {
                    correction |= 0x60;
                    set_flag_c = 0b0001_0000;
                }

                if n_flag {
                    value = value.overflowing_sub(correction).0
                } else {
                    value = value.overflowing_add(correction).0
                };

                value &= 0xff;

                let set_flag_z = if value == 0 { 0b1000_0000 } else { 0 };

                self.f &= !0b1011_0000;
                self.f |= set_flag_c | set_flag_z;

                self.a = value;
            }
            SCF => {
                self.set_flags(self.get_flag_bit(FLAG_Z) == 1, false, false, true);
            }
            CCF => {
                self.set_flags(self.get_flag_bit(FLAG_Z) == 1, false, false, !(self.get_flag_bit(FLAG_N) == 1));
            }
            CPL => {
                self.a = self.a ^ 0xFF;
                self.set_flag(FLAG_N, true);
                self.set_flag(FLAG_H, true);
            }
            RLCA | RLA => {
                let b7 = (self.a & 0x80) >> 7;
                let c = self.get_flag_bit(FLAG_C);
                self.a = self.a << 1;
                self.set_flags(false, false, false, b7 == 1);

                match instr {
                    RLA => self.a |= c,
                    RLCA => self.a |= b7,
                    _ => (),
                }
            }
            RRCA | RRA => {
                let b = self.a & 0x01;
                let c = (self.f & 0x10) >> 4;

                self.a = self.a >> 1;
                self.set_flags(false, false, false, b == 1);

                match instr {
                    RRA => self.a |= c << 7,
                    RRCA => self.a |= b << 7,
                    _ => (),
                }
            }
            CB => {
                let cb_opcode = self.fetch(memory);
                let prefix_instr = Instruction::decode_cb(cb_opcode);
                self.execute_prefix(prefix_instr, memory);
            }

            // Control flow
            JPnn => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                self.pc = u8_to_u16(msb, lsb);
            }
            JPHL => {
                self.pc = self.get_reg_view(HL);
            }
            JPccnn(cc) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);

                if self.check_conditions(cc) {
                    self.pc = nn;
                }
            }
            JRe => {
                let e = self.fetch(memory) as i8 as i16;
                self.pc = (self.pc as i16 + e) as u16;
            }
            JRcce(cc) => {
                let e = self.fetch(memory) as i8 as i16;
                if self.check_conditions(cc) {
                    self.pc = self.pc.overflowing_add_signed(e).0;
                }
            }
            CALLnn | CALLccnn(_) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);

                let cc = match instr {
                    CALLccnn(cc) => self.check_conditions(cc),
                    _ => true,
                };
                if cc {
                    self.sp = self.sp.overflowing_sub(1).0;
                    self.write(self.sp, high_u16(self.pc), memory);
                    self.sp = self.sp.overflowing_sub(1).0;
                    self.write(self.sp, low_u16(self.pc), memory);

                    self.pc = nn;
                }
            }
            RSTn(n) => {
                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, high_u16(self.pc), memory);
                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, low_u16(self.pc), memory);

                self.pc = RST[n as usize] as u16;
            }
            RET | RETcc(_) | RETI => {
                let cc = match instr {
                    RETcc(cc) => self.check_conditions(cc),
                    _ => true,
                };
                if cc {
                    let lsb = memory.read(self.sp);
                    self.sp = u16_add(self.sp, 1).0;
                    let msb = memory.read(self.sp);
                    self.sp = u16_add(self.sp, 1).0;
                    //println!("msb: {msb:#0X}, lsb: {lsb:#0X}");
                    match instr {
                        RETI => self.ime = 1,
                        _ => (),
                    }

                    self.pc = u8_to_u16(msb, lsb);
                }
            }
            INTn(nn) => {
                self.ime = 0;

                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, high_u16(self.pc), memory);
                self.sp = self.sp.overflowing_sub(1).0;
                self.write(self.sp, low_u16(self.pc), memory);

                self.pc = nn;
            }

            _ => panic!("Instruction not matched {:?}", instr),
        }
    }

    /*
     * CB Prefix instructions
     */

    /// Execute the decoded prefix instruction
    fn execute_prefix(&mut self, instr: Instruction, memory: &mut Memory) {
        match instr {
            RLCr(r) | RRCr(r) | RLr(r) | RRr(r) | SLAr(r) | SRAr(r) | SRLr(r) => {
                let mut rv = self.get_reg(r, memory);
                let (new_bit_loc, b) = match instr {
                    RLCr(_) | RLr(_) | SLAr(_) => (0, (rv & 0x80) >> 7),
                    RRCr(_) | RRr(_) | SRAr(_) | SRLr(_) => (7, rv & 0x01),
                    _ => (0, 0),
                };

                match instr {
                    RLCr(_) | RLr(_) | SLAr(_) => rv = rv << 1,
                    RRCr(_) | RRr(_) | SRAr(_) | SRLr(_) => rv = rv >> 1,
                    _ => (),
                }

                let c = (self.f & 0x10) >> 4;

                let new_bit = match instr {
                    RLCr(_) | RRCr(_) => b == 1,
                    RLr(_) | RRr(_) => c == 1,
                    SRAr(_) => (rv & 0x40) != 0,
                    SRLr(_) | SLAr(_) => false,
                    _ => panic!("nooooooo"),
                };

                rv = set_bit(rv, new_bit_loc, new_bit);

                self.set_reg(r, rv, memory);
                self.set_flags(rv == 0, false, false, b == 1);
            }
            SWAPr(r) => {
                let rv = self.get_reg(r, memory);
                let n_msb = low_u8(rv) << 4;
                let n_lsb = high_u8(rv);
                self.set_reg(r, n_msb + n_lsb, memory);
                self.set_flags(rv == 0, false, false, false);
            }
            BITnr { n, r } => {
                let reg_value = self.get_reg(r, memory);
                let bit = reg_value & (0x1 << n);
                self.set_flag(FLAG_Z, bit == 0);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, true);
            }
            RESnr { n, r } => self.set_reg(r, set_bit(self.get_reg(r, memory), n, false), memory),
            SETnr { n, r } => self.set_reg(r, set_bit(self.get_reg(r, memory), n, true), memory),
            ErrInstr { opcode } => panic!("Should not be here lol {:#0X}", opcode),
            _ => panic!("also should not be here lmfao"),
        }
    }

    /*
     * Interrupts
     */

    /// Handle Interrupts
    fn handle_interrupt(&mut self, memory: &mut Memory) -> bool {
        let (if_reg, ie_reg) = (memory.read(IF), memory.read(IE));

        if if_reg & 0b1 > 0 && ie_reg & 0b1 > 0 {
            //println!("VBlank interrupt");
            memory.write(IF, if_reg & 0b1111_1110);
            self.execute(INTn(0x0040), memory);
            return true;
        }
        if if_reg & 0b10 > 0 && ie_reg & 0b10 > 0 {
            println!("STAT interrupt");
            self.execute(INTn(0x0048), memory);
            memory.write(IF, if_reg & 0b1111_1101);
            return true;
        }
        if if_reg & 0b100 > 0 && ie_reg & 0b100 > 0 {
            println!("Timer interrupt");
            self.execute(INTn(0x0050), memory);
            memory.write(IF, if_reg & 0b1111_1011);
            return true;
        }
        if if_reg & 0b1000 > 0 && ie_reg & 0b1000 > 0 {
            println!("Serial interrupt");
            self.execute(INTn(0x0058), memory);
            memory.write(IF, if_reg & 0b1111_0111);
            return true;
        }
        if if_reg & 0b10000 > 0 && ie_reg & 0b10000 > 0 {
            println!("Joypad interrupt");
            self.execute(INTn(0x0060), memory);
            memory.write(IF, if_reg & 0b1110_1111);
            return true;
        }

        false
    }

    ///attempt to decompile the instructions or something (its literally just running, meaning it
    ///doesnt work at all)
    pub fn decompile(&mut self, memory: &mut Memory) {
        let data = memory.get_data();
        while (self.pc as usize) < data.len() {
            let opcode = self.fetch(memory);
            let instr = Instruction::decode(opcode);
            self.execute(instr, memory);
            println!("{:?}", instr);
        }
    }

    /*
     * Flag functions
     */
    
    pub fn get_flag_bit(&self, flag_bit: u8) -> u8 {
        let bit = 0b1 << flag_bit;
        (self.f & bit) >> flag_bit
    }

    /// Get the flag register
    pub fn get_flag(&self) -> u8 {
        self.f
    }

    /// Sets a specific flag in the flag register
    ///
    /// `flag_bit` - the bit of the flag in the register
    /// `value` - value that the bit should be set to
    fn set_flag(&mut self, flag_bit: u8, value: bool) {
        let bit = 0b1 << flag_bit;
        match value {
            true => self.f |= bit,
            false => self.f &= !bit,
        }
    }

    /// Set all the flags
    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_flag(FLAG_Z, z);
        self.set_flag(FLAG_N, n);
        self.set_flag(FLAG_H, h);
        self.set_flag(FLAG_C, c);
    }

    ///Sets the carry flags
    fn set_carry_flags(&mut self, carry: bool, half_carry: bool) {
        self.f = 0;
        if carry {
            self.f |= 0b0001_0000;
        } else {
            self.f &= 0b1110_1111;
        }
        if half_carry {
            self.f |= 0b0010_0000;
        } else {
            self.f &= 0b1101_1111;
        }
    }

    // Checks the specified condition in the flag register (zero flag or carry flag)
    pub fn check_conditions(&self, cc: u8) -> bool {
        let flag_z = self.f & 0b1000_0000 != 0;
        let flag_c = self.f & 0b0001_0000 != 0;
        //println!("{flag_z}, {cc}");
        //println!("{flag_c}, {cc}");
        //JP NZ = 0b1100_0010
        //JP NC = 0b1101_0010
        //JP Z = 0b1100_1010
        //JP C = 0b1101_1010
        match cc {
            0b00 => !flag_z,
            0b01 => flag_z,
            0b10 => !flag_c,
            0b11 => flag_c,
            _ => false,
        }
    }

    /*
     * 16-bit register view setting/getting
     */

    /// Get the register view for loading/storing to the address in the view
    pub fn get_reg_view_addr(&mut self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg_view(BC),
            DE => self.get_reg_view(DE),
            0x2 => {
                let r = self.get_reg_view(HL);
                self.set_rr(HL, u16_add(r, 1).0);
                r
            }
            0x3 => {
                let r = self.get_reg_view(HL);
                self.set_rr(HL, u16_sub(r, 1).0);
                r
            }
            _ => 0x00,
        }
    }

    /// Gets the register view or the stack pointer
    pub fn get_reg_view(&self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg2(B, C),
            DE => self.get_reg2(D, E),
            HL => self.get_reg2(H, L),
            SP => self.sp,
            _ => 0x00,
        }
    }

    pub fn get_reg_view_int(&self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg2(B, C),
            DE => self.get_reg2(D, E),
            HL => self.get_reg2(H, L),
            AF => self.get_reg2(A, F),
            _ => 0x00,
        }
    }

    /// Gets a register view
    fn get_reg2(&self, x: u8, y: u8) -> u16 {
        u8_to_u16(self.get_reg_int(x), self.get_reg_int(y))
    }

    /// Sets the specified registers with a u16
    fn set_reg2(&mut self, high: u8, low: u8, nn: u16) {
        self.set_reg_int(high, (nn >> 8) as u8);
        self.set_reg_int(low, nn as u8);
    }

    // Sets the register views or stack pointer
    fn set_rr(&mut self, rr_key: u8, nn: u16) {
        match rr_key {
            BC => self.set_reg2(B, C, nn),
            DE => self.set_reg2(D, E, nn),
            HL => self.set_reg2(H, L, nn),
            SP => self.sp = nn,
            _ => (),
        }
    }

    // Sets the internal register views
    fn set_reg_view_int(&mut self, rr_key: u8, nn: u16) {
        match rr_key {
            BC => self.set_reg2(B, C, nn),
            DE => self.set_reg2(D, E, nn),
            HL => self.set_reg2(H, L, nn),
            AF => self.set_reg2(A, F, nn),
            _ => (),
        }
    }

    /*
     * 8-bit register setting/getting
     */

    /// Sets the internal registers or reads HL
    fn set_reg(&mut self, r: u8, n: u8, memory: &mut Memory) {
        match r {
            B => self.b = n,
            C => self.c = n,
            D => self.d = n,
            E => self.e = n,
            H => self.h = n,
            L => self.l = n,
            A => self.a = n,
            READ_HL => {
                let loc = self.get_reg_view(HL);
                self.write(loc, n, memory);
            }
            _ => (),
        }
    }

    /// Sets the internal registers
    fn set_reg_int(&mut self, r: u8, n: u8) {
        match r {
            B => self.b = n,
            C => self.c = n,
            D => self.d = n,
            E => self.e = n,
            H => self.h = n,
            L => self.l = n,
            A => self.a = n,
            F => self.f = n & 0xF0,
            _ => (),
        }
    }

    /// Gets the internal registers
    pub fn get_reg_int(&self, reg: u8) -> u8 {
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,
            F => self.f,
            _ => 0,
        }
    }

    // Gets the internal registers or read from HL
    pub fn get_reg(&self, reg: u8, memory: &Memory) -> u8 {
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,
            READ_HL => {
                let loc = self.get_reg_view(HL);
                memory.read(loc)
            }
            _ => 0,
        }
    }

    /// Internal cpu write, redundant
    fn write(&mut self, addr: u16, byte: u8, memory: &mut Memory) {
        if addr == 0xFF44 {
            ()
        }
        //println!("wrote {0:02X} at address {1:04X}", byte, addr);
        memory.write(addr, byte);
    }

    pub fn print(&self) {
        println!("{:#?}", self);
    }
}

impl core::fmt::Debug for SharpSM83 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("SharpSM83")
            .field("pc", &format_args!("{:04X}", self.pc))
            .field("a", &format_args!("{:02X}", self.a))
            .field("b", &format_args!("{:02X}", self.b))
            .field("c", &format_args!("{:02X}", self.c))
            .field("d", &format_args!("{:02X}", self.d))
            .field("e", &format_args!("{:02X}", self.e))
            .field("h", &format_args!("{:02X}", self.h))
            .field("l", &format_args!("{:02X}", self.l))
            .field("sp", &format_args!("{:04X}", self.sp))
            .field("f", &format_args!("{:#010b}", self.f))
            .finish()
    }
}
