use crate::bytes::*;
use crate::memory::*;
use std::error::Error;
use std::fmt;

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

const XOFFSET: u8 = 3;
const YOFFSET: u8 = 0;
const REGMASK: u8 = 0b111u8;

const B: u8 = 0x0;
const C: u8 = 0x1;
const D: u8 = 0x2;
const E: u8 = 0x3;
const H: u8 = 0x4;
const L: u8 = 0x5;
const A: u8 = 0x7;
const READ_HL: u8 = 0x6;
const BC: u8 = 0x0;
const DE: u8 = 0x1;
const HL: u8 = 0x2;
const SP: u8 = 0x3;

// the cpu
pub struct SharpSM83 {
    pub mem_write_stack: Vec<(u8, u16)>,
    pub mem_write: u8,

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

impl SharpSM83 {
    pub fn run(&mut self, gamepack: &Memory) -> Result<(), FailedCPUInstruction> {
        println!("\n\nNEW INSTRUCTION STARTING");
        let result = match self.decode(gamepack) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("Error: {e}");
                Err(FailedCPUInstruction{})
            }
        };
        self.print_info();

        result
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

            // Z N H C 0 0 0 0
            f: 0b0000_0000,
            ime: 0b0,

            pc: 0x0,
            sp: 0x0000,

        }
    }

    fn fetch(&mut self, gamepack: &Memory) -> u8 {
        // ---- get instruction from memory  ----
        let opcode = gamepack.read(self.pc);
        println!("fetched instruction: {:#010b} at pc: {1}", opcode, self.pc);
        self.pc += 1;

        opcode
    }
    
    // REDO SO THAT EACH INSTRUCTION IS 8 BITS AND THE NEXT Immediate/etc valueS CAN BE FOUND FROM
    // INCREMENTING THE PC
    fn decode(&mut self, gamepack: &Memory) -> Result<(), OverflowError> {

        let opcode = self.fetch(gamepack);
        let _op_x = opcode >> 6;
        let _op_y = (opcode & 0b00111000) >> 3;
        let _op_z = opcode & 0b00000111;
        let _op_p = _op_y >> 1;
        let cc = _op_y & 0b100;
        let _op_q = _op_y % 2;

        let high = high_u8(opcode);
        let low = low_u8(opcode);
 
        if opcode == 0x00 {
            println!("NOP");
        } else if opcode == 0x10 {
            println!("Stop");
        } else if opcode == 0x76 {
            // 0b0111_0110 -> x = 110, y = 110
            println!("halt");
        } else if opcode == 0xF3 {
            self.ime = 0;
            println!("DI");
        } else if opcode == 0xFB {
            self.ime = 1;
            println!("EI");
        }
        // 8-bit load / store instructions

        // load / store to view address
        else if low_u8(opcode) == 0xA || low_u8(opcode) == 0x2 && high_u8(opcode) < 0x4 {
            println!("load / store to view");
            if high_u8(opcode) < 0x4 {
                let loc = self.get_rr(high_u8(opcode));

                if low_u8(opcode) == 0xA {
                    let n = self.read(loc, gamepack);
                    self.set_reg(A, n);
                } else {
                    self.write(loc, self.get_reg(A));
                }

                if high_u8(opcode) == 0x3 {
                    self.l -= 1;
                }else{
                    self.l += 1;
                }
            }
        }
        // ldh and load / store to (nn)
        else if high_u8(opcode) >= 0xE && (low_u8(opcode) == 0xA || low_u8(opcode) == 0x2 || low_u8(opcode) == 0x0) {
            println!("ldh and load/store to (nn)");
            let mut loc = u8_to_u16(0xFF, self.get_reg(C));

            if low_u8(opcode) == 0xA {
                let lsb = self.fetch(gamepack);
                let msb = self.fetch(gamepack);
                loc = u8_to_u16(lsb, msb);
            } else if low_u8(opcode) == 0x2 {
                loc = u8_to_u16(0xFF, self.get_reg(C));
            } else if low_u8(opcode) == 0x0 {
                loc = u8_to_u16(0xFF, self.fetch(gamepack));
            }

            if high_u8(opcode) == 0xE {
                self.write(loc, self.get_reg(A));
            } else if high_u8(opcode) == 0xF {
                let n = self.read(loc, gamepack);
                self.set_reg(A, n);
            }
        }
        // load r, r' / load r, n
        else if opcode >> 6 == 0b01 || 
            ((opcode < 0b0100_0000) && (low_u8(opcode) == 0x6 || low_u8(opcode) == 0xE)) {
            println!("generic load");
            let x = (opcode >> XOFFSET) & REGMASK;
            let y = (opcode >> YOFFSET) & REGMASK;

            if x == READ_HL && y == READ_HL && opcode >= 0b0100_0000 {
                println!("halt");
            } else {
                
                let n = if opcode < 0x40 {
                    self.fetch(gamepack)
                }else {
                    self.get_reg_or_mem(y, gamepack)
                };

                self.set_reg(x, n);
            }
        }
        // 16-bit load / store instructions
        // LDRRNNIMM    = 0b00xx0001
        // LDSPNN       = 0b00001000

        //load reg view with immediate
        else if low_u8(opcode) == 0x1 && opcode >> 6 == 0b00 {
            println!("load reg view with imm");
            let rr_key = opcode >> 4;
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            let nn = u8_to_u16(lsb, msb);
            self.load_rr(rr_key, nn);
        }
        // write to memory from sp
        else if opcode == 0x08 {
            println!("write to mem from sp");
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            let loc = u8_to_u16(lsb, msb);
            self.write(loc, high_u16(self.sp));
            self.write(loc + 1, low_u16(self.sp));
        }
        // load stack pointer from HL
        else if opcode == 0xF9 {
            println!("load sp from hl");
            self.load_rr(SP, self.get_reg_view(H, L));
        } 
        else if opcode == 0xF8 {
            println!("idk what this is yet");
            let e = self.fetch(gamepack);
            let result = i16_add(self.f as i16, e as i16);
            self.sp = result.0;
            if result.1 {
                self.f |= 0b1000_0000;
            }else{
                self.f &= 0b0111_1111;
            }
            if result.2 {
                self.f |= 0b0000_1000;
            }else{
                self.f &= 0b1111_0111;
            }
            self.load_rr(SP, self.get_reg_view(H, L)); // needs add u16
        }
        //stack push and pop
        else if _op_x == 0b11 && (low == 0b0101 || low == 0b0001) {
            println!("Push and pop");
            let rr_key = _op_p;            
            //push to stack 0b11xx0101
            if low == 0b0101 {
                let rr = self.get_rr(rr_key);
                let lsb = high_u16(rr);
                let msb = low_u16(rr);
                self.sp -= 1;
                self.write(self.sp, msb);
                self.sp -= 1;
                self.write(self.sp, lsb);
            }
            //pop from stack 0b11xx0001
            else if opcode & 0b0001 == 0b0001 {
                let lsb = self.read(self.sp, &gamepack);
                self.sp += 1;
                let msb = self.read(self.sp, &gamepack);
                self.sp += 1;

                self.load_rr(rr_key, u8_to_u16(lsb, msb));
            }
        }

        //-----------ARITHMETIC---------        

        //add, sub, adc, subc, and, or, xor, cp 
        else if (high >= 0x8 && high <= 0xB) || (high & 0b1100_0000 != 0 && (low == 0x5|| low == 0xE)) {
            println!("arithmetic with register");
            let x = opcode & REGMASK;
            let operation = high & 0x3;

            let n = if low == 0x5 || low == 0xE {
                self.fetch(gamepack)
            }else{
                self.get_reg(x)
            };

            let result = if low < 0x8 {
                match operation {
                    0 => u8_add(self.a, n),
                    1 => u8_sub(self.a, n),
                    2 => u8_and(self.a, n),
                    3 => u8_or(self.a, n),
                    _ => (0, 0),
                }
            }else {
                match operation {
                    0 => u8_add(self.a, n + self.f & 0b0010_000),
                    1 => u8_sub(self.a, n + self.f & 0b0010_000),
                    2 => u8_xor(self.a, n),
                    3 => u8_cmp(self.a, n),
                    _ => (0, 0),
                }
            };

            if low < 0x8 || operation != 3 {
                self.set_reg(A, result.0);
            }

            self.f = low_u8(self.f) + result.1;
        }

        else if opcode == 0xE8 {
            println!("0xE8");
            let e = self.fetch(gamepack);
            let result = i16_add(self.sp as i16, e as i16);
            self.sp = result.0;
            if result.1 {
                self.f |= 0b1000_0000;
            }else{
                self.f &= 0b0111_1111;
            }
            if result.2 {
                self.f |= 0b0000_1000;
            }else{
                self.f &= 0b1111_0111;
            }
        }

        // increment / decrement registers
        //0011 inc
        //0100 inc
        //0101 dec
        //1011 dec
        //1100 inc
        //1101 dec
        else if  _op_x == 0 && (_op_z == 3 || _op_z == 4 || _op_z == 5) {
            println!("increment");
            let x = (opcode >> 3) & 0b111;
            let is_inc = !cmpbit(low, 0b0001) || (!cmpbit(low, 0b1000) && cmpbit(low, 0b0100));
            let is_view = low == 0x3 || low == 0xB;
            match is_view {
                false => {
                    let r = self.get_reg(x);

                    let result = match is_inc {
                        true => u8_add(r, 1),
                        false => u8_sub(r, 1),
                    };

                    self.set_reg(x, result.0);
                },
                true => {
                    // this is probably a mix of 2 subbing from the lsb register and then checking
                    // the carry bit for  the msb register
                    let r = self.get_rr(high);

                    let result = match is_inc {
                        true => r + 1,
                        false => r - 1,
                    };

                    self.load_rr(high, result);
                },
            };
        } 

        // ccf, scf
        else if _op_x == 0 && _op_z == 7 && (_op_y == 6 || _op_y == 7){
            println!("ccf, scf");
            self.f = self.f & 0b1001_1111;

            if cmpbit(low, 0b1111){
                self.f &= 0b1110_1111;
            }else {
                self.f &= !(self.f & 0b1110_1111);//this might not work have to double
                                                  //check
            }
        }

        // daa
        // wtf is a daa decimal adjust accumulator
        else if opcode == 0x27 {
            println!("daa");
            let n_flag = self.f & 0b0100_0000;
            let h_flag = self.f & 0b0010_0000;
            let c_flag = self.f & 0b0001_0000;

            match n_flag {
                0 => {
                    if c_flag != 0 || self.a > 0x99 { self.a += 0x60; self.f |= 0b0100_0000; }
                    if h_flag != 0 || (self.a & 0x0F) > 0x09  {self.a += 0x6; }
                },
                _ => {
                    if c_flag != 0 { self.a -= 0x60; }
                    if h_flag != 0 { self.a -= 0x6; }
                },
            }

            if self.a == 0 { self.f |= 0b1000_0000 }
            else { self.f &= 0b0111_1111 };

            self.f &= 0b1101_1111;
        }

        //cpl
        else if opcode == 0x2F {
            println!("cpl");
            self.a = !self.a;
            self.f |= 0b0110_0000;
        }


        // ROTATES AND SHIFTS
        // rlca and rla
        else if opcode == 0x07 || opcode == 0x17 {
            println!("rotate left");
            let r = self.a;

            let fill = if high == 0x1 {
                r & 0b1000_0000
            }else {
                self.f & 0b0001_0000
            };
            
            self.a <<= 1;

            self.a = match fill != 0 {
                true => self.a | 0b0000_0001,
                false => self.a & 0b1111_1110
            }; 

            self.f = match fill != 0 {
                true => self.f | fill,
                false => self.f & !fill
            }; 
        }

        // rrca and rra
        else if opcode == 0x0F || opcode == 0x1F {
            println!("rotate right");
            let r = self.a;

            let fill = if high == 0x1 {
                r & 0b0000_0001
            }else {
                self.f & 0b0001_0000
            };
            
            self.a >>= 1;

            self.a = match fill != 0 {
                true => self.a | 0b1000_0000,
                false => self.a & 0b0111_1111
            }; 

            self.f = match fill != 0 {
                true => self.f | fill,
                false => self.f & !fill
            }; 
        }
        
        else if opcode == 0xCB {
            println!("PREFIX CB OR SOMETHING LIKE THAT");
        }

        //--------CONTROL FLOW---------

        //Jump nn
        else if opcode == 0b1100_0011 {
            println!("JP nn");
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            self.pc = u8_to_u16(lsb, msb);
        }
        //jump HL
        else if opcode == 0b11101001 {
            println!("JP HL");
            self.pc = self.get_rr(HL);
        }
        //jump cc, nn
        else if opcode & 0b010 != 0 && opcode & 0b11000000 != 0 {
            println!("JP cc, nn");
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            let nn = u8_to_u16(lsb, msb);

            if self.check_conditions(cc) {
                self.pc = nn;
            }
        }
        
        // JR e
        else if opcode == 0x18 {
            println!("JR e");
            let e = self.fetch(gamepack) as i16;
            self.pc = (self.pc as i16 + e as i16) as u16;
        }
        
        //jump cc, e
        else if opcode & 0b0000_0111 != 0 && opcode & 0b1110_0000 == 0b0010_0000 {
            println!("JP cc, e");
            let e = self.fetch(gamepack);

            if self.check_conditions(cc) {
                self.pc = (self.pc as i16 + e as i16) as u16;
            }
        }
        

        // CALL nn
        else if opcode == 0xCD {
            println!("CALL nn");
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            let nn = u8_to_u16(lsb, msb);
            
            self.sp -= 1;
            self.write(self.sp, msb);
            self.sp -= 1;
            self.write(self.sp, lsb);
            
            self.pc = nn;
        }
        
        //CALL cc, nn
        else if opcode & 0b0000_0100 != 0 && opcode & 0b1100_0000 != 0 {
            println!("CALL cc, nn");
            let lsb = self.fetch(gamepack);
            let msb = self.fetch(gamepack);
            let nn = u8_to_u16(lsb, msb);

            self.sp -= 1;
            self.write(self.sp, msb);
            self.sp -= 1;
            self.write(self.sp, lsb);

            if self.check_conditions(cc) {
                self.pc = nn;
            }
        }

        //RET
        else if opcode == 0xC9 {
            println!("RET");
            let lsb = self.read(self.sp, gamepack);
            self.sp += 1;
            let msb = self.read(self.sp, gamepack);
            self.sp += 1; 
            self.pc = u8_to_u16(lsb, msb);
            panic!("return");
        }

        //RET cc
        else if _op_z == 0b000 && opcode & 0b1100_0000 != 0  {
            println!("RET cc");
            if self.check_conditions(cc){
                let lsb = self.read(self.sp, gamepack);
                self.sp += 1;
                let msb = self.read(self.sp, gamepack);
                self.sp += 1; 
                self.pc = u8_to_u16(lsb, msb);
            }

            panic!("return");
        }

        //RETI
        else if opcode == 0xD9 {
            println!("RETI");
            let lsb = self.read(self.sp, gamepack);
            self.sp += 1;
            let msb = self.read(self.sp, gamepack);
            self.sp += 1; 
            self.pc = u8_to_u16(lsb, msb);
            self.ime = 1;
        }
        
        //RST
        else if _op_x == 0b11 && _op_z == 0b111 {
            println!("RST");
            let n = _op_y;
            self.sp -= 1;
            self.write(self.sp, high_u16(self.pc));
            self.sp -= 1;
            self.write(self.sp, low_u16(self.pc));
            
            self.pc = u8_to_u16(n, 0x00);

        }

        Ok(())
    }

    fn check_conditions(&self, cc: u8) -> bool {
        let flag_z = self.f & 0b1000_0000 != 0;
        let flag_c = self.f & 0b0001_0000 != 0;

        match cc {
            0 => !flag_z,
            1 => flag_z,
            2 => !flag_c,
            3 => flag_c,
            _ => false
        }


    }

    fn set_rr_from_u16(&mut self, r1: u8, r2: u8, nn: u16) {
        self.set_reg(r1, (nn >> 8) as u8);
        self.set_reg(r2, nn as u8);
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

    pub fn get_reg_or_mem(&self, reg: u8, gamepack: &Memory) -> u8 {
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
                self.read(loc, gamepack)
            }
            _ => 0,
        }
    }

    fn read(&self, addr: u16, gamepack: &Memory) -> u8 {
        gamepack.read(addr)
    }

    fn write(&mut self, addr: u16, reg: u8) {
        println!("wrote {0:02X} at address {1:04X}", reg, addr);
        self.mem_write_stack.push((reg, addr));
        self.mem_write += 1;
    }

    pub fn print_info(&self) {
        println!("{:#?}", self);
    }
}

