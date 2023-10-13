pub mod instruction;
pub mod identifiers;

use crate::console::regids::*;
use crate::cpu::instruction::*;
use crate::cpu::identifiers::*;

use Instruction::*;

use crate::bytes::*;
use crate::memory::*;

use std::error::Error;
use std::fmt;
use std::time::Instant;


#[derive(Debug)]
pub struct FailedCPUInstruction{
}

impl Error for FailedCPUInstruction {}

impl fmt::Display for FailedCPUInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed cpu instruction, aborting")
    }
}

#[derive(Debug)]
struct OverflowError;
impl Error for OverflowError {}

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Detected Stack Smashing or adding with overflow")
    }
}

#[derive(Debug)]
struct InvalidRegError;

impl Error for InvalidRegError {}

impl fmt::Display for InvalidRegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid specified register")
    }
}

pub enum Mode {
    EMU,
    CPU,
}

// the cpu
pub struct SharpSM83 {
    pub mem_write_stack: Vec<(u8, u16)>,
    pub mem_write: u8,

    machine_cycles: u32,

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
    pub mode: Mode,

    pub verbose: bool,

}

impl SharpSM83 {
    pub fn is_interruptible(&self) -> bool {
        self.ime == 1
    }
    pub fn new() -> SharpSM83 {
        SharpSM83 {
            mem_write_stack: Vec::new(),
            mem_write: 0,

            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            f: 0x00, // Z N H C 0 0 0 0

            machine_cycles: 0,

            ime: 0b0,

            pc: 0x0000,
            sp: 0x0000,

            stop: false,
            halt: false,
            mode: Mode::CPU,
            verbose: true,
        }
    }
    
    pub fn set_verbose(&mut self, b: bool) -> &mut Self {
        self.verbose = b;
        self
    }

    pub fn run(&mut self, memory: &Memory) -> Result<(), FailedCPUInstruction> {

        let start = Instant::now();

        if self.ime > 0 {
            self.handle_interrupt(memory.read(IF), memory.read(IE));
        }

        let opcode = self.fetch(memory);
        let instr = self.decode(opcode);
        eprintln!("{:?}", instr);
        self.execute(instr, memory);

        let instr_time = (start.elapsed().as_nanos() as f64) / (10u32.pow(9) as f64);
        //eprintln!("cpu_speed: {:.5} hz", 1./instr_time);
        self.print_info();

        Ok(())
    }

    pub fn raw_run(&mut self, memory: &mut Memory) {
        let opcode = self.fetch(memory);
        let instr = self.decode(opcode);
        eprintln!("{:?}", instr);
        self.execute(instr, memory);
        self.print_info();

        while self.mem_write_stack.len() > 0 {
            self.mem_write -= 1;
            let data = self.mem_write_stack.pop();

            match data {
                Some((x, y)) => memory.write(y, x),
                _ => ()
            }
        }
    }

    fn fetch(&mut self, memory: &Memory) -> u8 {
        // ---- get instruction from memory  ----
        let opcode = memory.read(self.pc);

        eprintln!("fetched instruction: {0:#04X} at pc: {1:#04X}", opcode, self.pc);     

        self.pc = u16_add(self.pc, 1).0;

        opcode
    }


    fn decode(&mut self, opcode: u8) -> Instruction {
        let op_x = opcode >> 6;
        let op_y = (opcode & 0b00111000) >> 3;
        let op_z = opcode & 0b00000111;
        let op_p = op_y >> 1;
        let cc = op_y & 0b011;
        let op_q = op_y % 2;

        return if opcode == 0x00 {
            NOP
        } else if opcode == 0x10 {
            STOP
        } else if opcode == 0x76 {
            HALT
        } else if opcode == 0xF3 {
            DI
        } else if opcode == 0xFB {
            EI
        }
        // 8-bit load / store instructions registers
        // LD r r
        else if op_x == 0b01 {
            LDRwR(op_y, op_z)
        }
        // LD r n
        else if op_x == 0b00 && op_z == 0b110{
            LDRwN(op_y)
        }
        // LDH
        else if op_x == 0b11 && op_y & 0b001 == 0b000 && op_z & 0b101 == 0b000 {
            LDH(op_y == 0b100, op_z == 0b000)
        }
        // LD A (nn)
        else if op_x == 0b11 && op_y & 0b101 == 0b101 && op_z == 0b010 {
            LDAwNNa(op_y == 0b101)
        }
        // LD (rr) A, LD A (rr)
        else if op_x == 0b00 && op_z == 0b010{
            if op_y == 0b0 {
                LDAwRRa(op_p)
            }
            else {
                LDRRawA(op_p)
            }
        }

        // 16-bit loads
        // LD rr nn
        else if op_x == 0b00 && op_q == 0b0 && op_z == 0b001 {
            LDrrnn(op_p)
        }
        //LD (nn) SP
        else if opcode == 0x08 {
            LDNNawSP
        }
        // load stack pointer from HL
        else if opcode == 0xF9 {
            LDSPwHL
        } 
        else if opcode == 0xF8 {
            LDHLwSP
        }
        // push rr
        else if op_x == 0b11 && op_q == 0 && op_z == 0b101{
            PUSHrr(op_p)
        }
        // pop rr
        else if op_x == 0b11 && op_q == 0 && op_z == 0b001{
            POPrr(op_p)
        }

        //-----------ARITHMETIC---------        

        //add, sub, adc, subc, and, or, xor, cp 
        else if op_x == 0b10 && op_q == 0b0 {
            match op_p {
                0 => Add(op_z),
                1 => Sub(op_z),
                2 => And(op_z),
                3 => Or(op_z),
                _ => ErrInstr{opcode},
            }
        }
        else if op_x == 0b10 && op_q == 0b1 {
            match op_p {
                0 => Adc(op_z),
                1 => Sbc(op_z),
                2 => Xor(op_z),
                3 => Cmp(op_z),
                _ => ErrInstr{opcode},
            }
        }
        // arithmetic with n
        else if op_x == 0b11 && op_z == 0b110 {
            match op_y {
                0b000 => Addn,
                0b001 => Adcn,
                0b010 => Subn,
                0b011 => Sbcn,
                0b100 => Andn,
                0b101 => Xorn,
                0b110 => Orn,
                0b111 => Cmpn,
                _ => ErrInstr{opcode},
            }
        } 
        // increment / decrement registers
        else if op_x == 0b00 && op_z == 0b100 {
            IncR(op_p)
        }
        else if op_x == 0b00 && op_z == 0b101 {
            DecR(op_p)
        }

        // 16-bit arithmetic
        // ADD SP e
        else if opcode == 0xE8 {
            AddSpE
        }
        // ADD HL rr
        else if op_x == 0b00 && op_q == 0b1 && op_z == 0b001 {
            ADDHLrr(op_p)
        }
        // INC / DEC
        else if op_x == 0b00 && op_q == 0b0 && op_z == 0b011 {
            INCrr(op_p)
        }
        else if op_x == 0b00 && op_q == 0b1 && op_z == 0b011 {
            DECrr(op_p)
        }
        else if opcode == 0x27 {
            DAA
        }
        else if opcode == 0x37 {
            SCF
        }
        else if opcode == 0x2F {
            CPL
        }
        else if opcode == 0x3F {
            CCF
        }       

        // ROTATES AND SHIFTS & Bit Ops
        // RLCA
        else if opcode == 0x07 {
            RLCA
        }
        // RLA
        else if opcode == 0x17 {
            RLA
        }
        // RRCA
        else if opcode == 0x0F {
            RRCA
        }
        // RRA
        else if opcode == 0x1F {
            RRA
        }
        // prefix CB
        else if opcode == 0xCB {
            CB
        }

        //--------CONTROL FLOW---------

        //Jump nn
        else if opcode == 0b1100_0011 {
            JPnn
        }
        //jump HL
        else if opcode == 0b1110_1001 {
            JPHL
        }
        //jump cc, nn
        else if op_x == 3 && op_y & 0b100 == 0b000 && op_z == 2 {
            JPccnn(cc)
        }
        // JR e
        else if opcode == 0x18 {
            JRe
        }

        //JR cc, e
        else if op_x == 0b00 && op_y & 0b100 == 0b100 && op_z == 0b000 {
            JRcce(cc)
        }
        // CALL nn
        else if opcode == 0xCD {
            CALLnn
        }
        //CALL cc, nn
        else if op_x == 0b11 && op_y & 0b100 == 0b000 && op_z == 0b100 {
            CALLccnn(cc)
        }
        //RET
        else if opcode == 0xC9 {
            RET
        }
        //RET cc
        else if op_x == 0b11 && op_y & 0b100 == 0b000 && op_z == 0b000 {
            RETcc(cc)
        }
        //RETI
        else if opcode == 0xD9 {
            RETI
        }
        //RST n
        else if op_x == 0b11 && op_z == 0b111 {
            RSTn(op_y)
        }
        else {
            ErrInstr{opcode}
        }
    }

    pub fn execute(&mut self, instr: Instruction, memory: &Memory) {
        match instr {
            ErrInstr{opcode} => {
                match opcode {
                    0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => eprintln!("Instruction Undefined"),
                    _ => panic!("Invalid Instruction {opcode:#010b}"),
                }
            },
            NOP => (),
            STOP => self.stop = true,
            HALT => self.halt = true,
            DI => self.ime = 0,
            EI => self.ime = 1,
            LDRwR(r1, r2) => self.set_reg(r1, self.get_reg(r2)),
            LDRwN(r) => {
                let n = self.fetch(memory);
                self.set_reg(r, n);
            },
            LDH(from, n) => {
                let loc = u8_to_u16(0xFF, match n {
                    true => self.fetch(memory),
                    false => self.get_reg(C),
                });
                match from {
                    true => {self.write(loc, self.get_reg(A))}
                    false => {self.set_reg(A, self.read(loc, memory))}
                } 
            }
            LDAwNNa(from) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let loc = u8_to_u16(msb, lsb);
                match from {
                    false => {self.set_reg(A, self.read(loc, memory))}
                    true => {self.write(loc, self.get_reg(A))}
                } 
            },
            LDRRawA(rr) => {
                let loc = self.get_rr_s(rr);
                self.write(loc, self.get_reg(A));
            },
            LDAwRRa(rr) => {
                let loc = self.get_rr_s(rr);
                self.set_reg(A, self.read(loc, memory));
            },
            LDrrnn(rr) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);
                self.load_rr(rr, nn);
            },
            LDNNawSP => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let loc = u8_to_u16(msb, lsb);
                self.write(loc, low_u16(self.sp));
                self.write(loc + 1, high_u16(self.sp));
            },
            PUSHrr(rr) => {
                let rr = self.get_rr(rr);
                let lsb = low_u16(rr);
                let msb = high_u16(rr);
                self.sp -= 1;
                self.write(self.sp, msb);
                self.sp -= 1;
                self.write(self.sp, lsb);
            },
            POPrr(rr) => {
                let lsb = self.read(self.sp, &memory);
                self.sp += 1;
                let msb = self.read(self.sp, &memory);
                self.sp += 1;
                self.load_rr_views(rr, u8_to_u16(msb, lsb));
            },
            LDHLwSP => {
                let e = self.fetch(memory);
                let result = i16_add(self.get_rr(SP) as i16 , e as i8 as i16);
                self.load_rr(HL, result.0);
                self.set_carry_flags(result.1, result.2);
            }
            LDSPwHL => {
                self.load_rr(SP, self.get_reg_view(H, L));
            },
            Add(_) | Sub(_) | And(_) | Or(_) | Adc(_) | Sbc(_) | Xor(_) | Cmp(_) | Addn | Subn | Andn | Orn | Adcn | Sbcn | Xorn | Cmpn => {
                let n = match instr {
                    Addn | Subn | Andn | Orn | Adcn | Sbcn | Xorn | Cmpn => self.fetch(memory),
                    Add(r) | Sub(r) | And(r) | Or(r) | Adc(r) | Sbc(r) | Xor(r) | Cmp(r) => self.get_reg(r),
                    _ => panic!("not supposed to get here")
                };
                let result = match instr { //kinda yuck
                    Add(_) | Addn => u8_add(self.a, n),
                    Sub(_) | Subn => u8_sub(self.a, n),
                    And(_) | Andn => u8_and(self.a, n),
                    Or(_) | Orn => u8_or(self.a, n),
                    Adc(_) | Adcn => u8_add(self.a, u8_add(n, self.f & 0b0010_000).0),
                    Sbc(_) | Sbcn => u8_sub(self.a, u8_add(n, self.f & 0b0010_000).0),
                    Xor(_) | Xorn => u8_xor(self.a, n),
                    Cmp(_) | Cmpn => u8_cmp(self.a, n),

                    _ => (0, 0)
                };
                self.set_reg(A, result.0);
                self.f = low_u8(self.f) + result.1;    
            },
            AddSpE => {
                let e = self.fetch(memory);
                let result = i16_add(self.sp as i16, e as i8 as i16);
                self.sp = result.0;
                self.set_carry_flags(result.1, result.2);
            },
            INCrr(rr) | DECrr(rr) | ADDHLrr(rr) => {
                let rrv = self.get_rr(rr);
                let mut rr_key = rr;
                let result = match instr {
                    INCrr(_) => u16_add(rrv, 1),
                    DECrr(_) => u16_sub(rrv, 1),
                    ADDHLrr(_) => {
                        let hl = self.get_rr(HL);
                        rr_key = HL;
                        u16_add(hl, rrv)
                    }
                    _ => panic!("Should not be here")
                };

                self.load_rr(rr_key, result.0);

                self.set_carry_flags(result.1, result.2);
            },

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

                if n_flag{ value -= correction } else { value += correction };

                value &= 0xff;

                let set_flag_z = if value == 0 { 0b1000_0000} else { 0 };  

                self.f &= !0b1011_0000;
                self.f |= set_flag_c | set_flag_z;

                self.a = value;
            },
            SCF | CCF => {

                self.f = self.f & 0b1001_1111;
                let c_flag = match instr {
                    CCF => self.f & 0b0001_0000,
                    SCF => 0b0,
                    _ => panic!("should not be here (scf/ccf error)")
                };
                self.f = match c_flag != 0 {
                    true => self.f ^ 0b0001_0000,
                    false => self.f | 0b0001_0000,
                };
            },
            CPL => {
                self.a = !self.a;
                self.f |= 0b0110_0000;
            },
            RLCA | RLA => {

            },
            RRCA | RRA => {

            },
            CB => {
                self.cb_prefix(memory);
            },

            // Control flow
            JPnn => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                self.pc = u8_to_u16(msb, lsb);
            },
            JPHL => {
                self.pc = self.get_rr(HL);
            },
            JPccnn(cc) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);

                if self.check_conditions(cc) {
                    self.pc = nn;
                }
            },
            JRe => {
                let e = self.fetch(memory) as i16;
                self.pc = (self.pc as i16 + e as i8 as i16 ) as u16;
            },
            JRcce(cc) => {
                let e = self.fetch(memory);
                if self.check_conditions(cc) {
                    self.pc = (self.pc as i16 + e as i8 as i16) as u16;
                }
            },
            CALLnn | CALLccnn(_) => {
                let lsb = self.fetch(memory);
                let msb = self.fetch(memory);
                let nn = u8_to_u16(msb, lsb);

                let cc = match instr {
                    CALLccnn(cc) => self.check_conditions(cc),
                    _ => true
                };
                if cc {
                    self.sp = u16_sub(self.sp, 1).0;
                    self.write(self.sp, low_u16(self.pc));

                    self.sp = u16_sub(self.sp, 1).0;
                    self.write(self.sp, high_u16(self.pc));

                    self.pc = nn;
                }

            },
            RET | RETcc(_) | RETI | RSTn(_) => {
                let cc = match instr {
                    RETcc(cc) => self.check_conditions(cc),
                    _ => true
                };
                if cc {
                    let mut msb = self.read(self.sp, memory);
                    self.sp = u16_add(self.sp, 1).0;
                    let mut lsb = self.read(self.sp, memory);
                    self.sp = u16_add(self.sp, 1).0;

                    match instr {
                        RETI => self.ime = 1,
                        RSTn(n) => {
                            msb = RST[n as usize];
                            lsb = 0x00;
                        }
                        _ => ()
                    }

                    self.pc = u8_to_u16(msb, lsb);
                }

            },

            _ => println!("Instruction not matched"),
        }

    }

    fn set_carry_flags(&mut self, a: bool, b: bool) {
        self.f = 0;
        if a {
            self.f |= 0b0001_0000;
        }else{
            self.f &= 0b1110_1111;
        }
        if b {
            self.f |= 0b0010_0000;
        }else{
            self.f &= 0b1101_1111;
        }
    }

    pub fn quick_init(&mut self){
        self.a = 0;
        self.f = 0;
        self.b = 0xff;
        self.c = 0x13;
        self.d = 0;
        self.e = 0xC1;
        self.h = 0x84;
        self.l = 0x03;
        self.pc = 0x100;
        self.sp = 0xFFFE;
    }

    pub fn init_emu(&mut self) {
        self.mode = Mode::EMU;
    }

    fn handle_interrupt(&mut self, if_reg: u8, ie_reg: u8) {
        if if_reg & 0b1 > 0 && ie_reg & 0b1 > 0 {
            println!("VBlank interrupt");
        }
        if if_reg & 0b10 > 0 && ie_reg & 0b10 > 0 {
            println!("LCD interrupt");
        }
        if if_reg & 0b100 > 0 && ie_reg & 0b100 > 0 {
            println!("Timer interrupt");
        }
        if if_reg & 0b1000 > 0 && ie_reg & 0b1000 > 0 {
            println!("Serial interrupt");
        }
        if if_reg & 0b10000 > 0 && ie_reg & 0b10000 > 0 {
            println!("Joypad interrupt");
        }

    }

    fn cb_prefix(&mut self, memory: &Memory) -> Result<(), String> {
        let opcode = self.fetch(memory);
        let op_x = opcode >> 6;
        let op_y = (opcode & 0b00111000) >> 3;
        let op_z = opcode & 0b111;
        let op_p = op_y >> 1;
        let op_q = op_y & 0b1;

        let n = op_y;
        let r = self.get_reg_or_mem(op_z, memory);

        match op_x {
            0 => {
                //rotate register or memory, this one is more complicated
                // NOTE: for arithmetic shifts cast as i8
                let shift = if op_q == 0 {
                    Shift::LEFT
                } else {
                    Shift::RIGHT
                };

                match op_p {
                    0 => {panic!("should not be here lol")},
                    1 => {self.set_reg(op_z, shift.s_i8(r as i8, n) as u8)},
                    2 => {self.set_reg(op_z, shift.s_u8(r, n))},
                    3 => {
                        if op_q == 0 {
                            let hi = high_u8(r) >> 4;
                            let lo = low_u8(r);
                            self.set_reg(op_z, hi + lo << 4);
                        }else{
                            self.set_reg(op_z, shift.s_u8(r, n));
                        }
                    },
                    _ => panic!("should not be here lol")
                }

                //panic!("wow");
            },
            1 => {
                //test bit
                self.set_flag(FLAG_Z, bit!(r, n) != 0);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, true);
            },
            2 => {
                //ruset
                self.set_reg(op_z, set_bit(r, n, false));
            },
            3 => {
                //set
                self.set_reg(op_z, set_bit(r, n, true));
            },
            _ => panic!("how did i get here")
        }

        Ok(())
    }

    pub fn decompile(&mut self, memory: &Memory) {
        let data = memory.get_data();
        while (self.pc as usize) < data.len() {
            let opcode = self.fetch(memory);
            let instr = self.decode(opcode);
            self.execute(instr, memory);
            println!("{:?}", instr);
        }

    }

    pub fn check_conditions(&self, cc: u8) -> bool {
        let flag_z = self.f & 0b1000_0000 != 0;
        let flag_c = self.f & 0b0001_0000 != 0;
        //println!("{flag_z}, {cc}");

        match cc {
            0 => flag_z,
            1 => !flag_z,
            2 => flag_c,
            3 => !flag_c,
            _ => false
        }


    }

    fn set_flag(&mut self, flag_bit: u8, value: bool) {
        match value {
            true => {self.f |= 0b1 << flag_bit},
            false => {self.f &= !(0b1 << flag_bit)}
        }
    }

    pub fn get_flag(&self) -> u8 {
        self.f
    }

    fn set_rr_from_u16(&mut self, r1: u8, r2: u8, nn: u16) {
        self.set_reg(r1, (nn >> 8) as u8);
        self.set_reg(r2, nn as u8);
    }

    pub fn get_rr_s(&mut self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg_view(B, C),
            DE => self.get_reg_view(D, E),
            0x2 => {
                let r = self.get_reg_view(H, L);
                self.load_rr(HL, u16_add(r, 1).0);
                r
            },
            0x3 => {
                let r = self.get_reg_view(H, L);
                self.load_rr(HL, u16_sub(r, 1).0);
                r
            },
            _ => 0x00,
        }
    }

    pub fn get_rr(&self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg_view(B, C),
            DE => self.get_reg_view(D, E),
            HL => self.get_reg_view(H, L),
            SP => self.sp,
            _ => 0x00,
        }
    }

    fn load_rr(&mut self, rr_key: u8, nn: u16) {
        match rr_key {
            BC => self.set_rr_from_u16(B, C, nn),
            DE => self.set_rr_from_u16(D, E, nn),
            HL => self.set_rr_from_u16(H, L, nn),
            SP => self.sp = nn,
            _ => (),
        }
    }

    fn load_rr_views(&mut self, rr_key: u8, nn: u16) {
        match rr_key {
            BC => self.set_rr_from_u16(B, C, nn),
            DE => self.set_rr_from_u16(D, E, nn),
            HL => self.set_rr_from_u16(H, L, nn),
            3 => {
                self.set_reg(A, (nn >> 8) as u8);
                self.f = nn as u8;
            },
            _ => (),
        }
    }

    fn get_reg_view(&self, x: u8, y: u8) -> u16 {
        u8_to_u16(self.get_reg(x), self.get_reg(y))
    }

    fn set_reg(&mut self, r: u8, n: u8) {
        match r {
            B => self.b = n,
            C => self.c = n,
            D => self.d = n,
            E => self.e = n,
            H => self.h = n,
            L => self.l = n,
            A => self.a = n,
            READ_HL => {
                let loc = self.get_rr(HL);
                self.write(loc, n);
            }
            _ => (),
        }
    }

    pub fn get_reg(&self, reg: u8) -> u8 {
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,
            _ => 0,
        }
    }

    pub fn get_reg_or_mem(&self, reg: u8, memory: &Memory) -> u8 {
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,
            READ_HL => {
                let loc = self.get_reg_view(H, L);
                self.read(loc, memory)
            }
            _ => 0,
        }
    }

    fn read(&self, addr: u16, memory: &Memory) -> u8 {
        memory.read(addr)
    }

    fn write(&mut self, addr: u16, reg: u8) {
        println!("wrote {0:02X} at address {1:04X}", reg, addr);
        self.mem_write_stack.push((reg, addr));
        self.mem_write += 1;
    }

    pub fn print_info(&self) {
        match self.mode {
            Mode::CPU => println!("{:#?}", self),
            Mode::EMU => (),
        }
    }
}

impl core::fmt::Debug for SharpSM83 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("SharpSM83")
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
