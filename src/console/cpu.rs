use std::fmt;
use std::error::Error;
use crate::bytes::*;
use crate::memory::*;

const LDREG: u8 = 0b01;
const LDIMM: u8 = 0b110;

const XOFFSET: u8 = 3;
const YOFFSET: u8 = 0;
const REGMASK: u8 = 0b00000111;

pub const N: usize = 20;
const SCUFFED_INSTRUCTIONS: [u8; N] = 
[
    0b0010_0001, //load HL, nn
    0b0000_0000, // nn = 0002
    0b0000_0010,

    0b0011_0110, // load (HL), n
    0b1110_0011, // n = 0xE3

    0b1110_0101, // push HL
    0b1101_0001, // pop BC


    0b0010_1110,
    0b0011_1100, // ldimm
    0b0100_0101, // ldreg
    0b0100_1110, // ld hl
    0b0111_0000, // str hl
    0b0011_0110,
    0b0010_0110, // str hl imm
    0b0001_0001, // load 16
    0b0100_1111,
    0b1111_0010,
    0b0000_1000, //load from sp
    0b0000_0000,
    0b0000_0011,

];

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

    //16-bit special purpose
    pub pc: usize,
    sp: u16,

}

const B: u8 = 0x0;
const C: u8 = 0x1;
const D: u8 = 0x2;
const E: u8 = 0x3;
const H: u8 = 0x4;
const L: u8 = 0x5;
const A: u8 = 0x7;
const BC: u8 = 0x0;
const DE: u8 = 0x1;
const HL: u8 = 0x2;
const SP: u8 = 0x3;

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

    pub fn run(&mut self, gamepack: &Memory) {
        println!("\n\nNEW INSTRUCTION STARTING");
        self.decode(gamepack);
        self.print_info();
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


            /* FLAG register
             * r - read
             * w - can be written
             * u - unimplemented
             * -n - value after system reset, 0 1 or x
             * x - depends on external things like user input
             */
            // r/w-0, r/w-1, u-1, r-0, r-1, r-x, w-1, u-0
            // value, -, bigval, flag, -
            f: 0b01_0_010_1_0, //specific value for the flags at boot i think

            pc: 0,
            sp: 0x0004,
        }

    }  

    fn fetch(&mut self) -> u8 {

        // ---- get instruction from memory  ----
        let opcode = SCUFFED_INSTRUCTIONS[self.pc];
        println!("fetched instruction: {:#010b} at pc: {1}", opcode, self.pc);
        self.pc += 1;

        opcode
    }
    

    // REDO SO THAT EACH INSTRUCTION IS 8 BITS AND THE NEXT Immediate/etc valueS CAN BE FOUND FROM
    // INCREMENTING THE PC
    fn decode(&mut self, gamepack: &Memory){

        let opcode = self.fetch();

        if opcode == 0 {
            ()
        }
        
        // refactor this to clean it up maybe instead of 100 if statements
        // 8-bit load / store instructions
        if opcode == 0x36 {
            let n = self.fetch();
            self.write(self.get_rr(HL), n);
        }

        else if opcode == 0x0A {
            let loc = self.get_reg_view(B, C); 
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
        } 
        else if opcode == 0x1A {
            let loc = self.get_reg_view(D, E); 
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
        }
        else if opcode == 0x02 {
            let loc = self.get_reg_view(B, C); 
            self.write(loc, A);
        }
        else if opcode == 0x12 {
            let loc = self.get_reg_view(D, E); 
            self.write(loc, A);
        }
        else if opcode == 0xFA {
            let lsb = self.fetch();
            let msb = self.fetch();
            let loc = u8_to_u16(lsb, msb);
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
        } 
        else if opcode == 0xEA {
            let lsb = self.fetch();
            let msb = self.fetch();
            let loc = u8_to_u16(lsb, msb);
            self.write(loc, A);
        } 
        else if opcode == 0xF2 {
            let loc = u8_to_u16(0xFF, self.get_reg(C));
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
        }
        else if opcode == 0xE2 {
            let loc = u8_to_u16(0xFF, self.get_reg(C));
            self.write(loc, A)
        }
        else if opcode == 0xF0 {
            let lsb = self.fetch(); 
            let loc = u8_to_u16(0xFF, lsb);
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
        }
        else if opcode == 0xF0 {
            let lsb = self.fetch(); 
            let loc = u8_to_u16(0xFF, lsb);
            self.write(loc, A);
        }
        else if opcode == 0x3A {
            let loc = self.get_reg_view(H, L);
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
            self.l -= 1;
        }
        else if opcode == 0x32 {
            let loc = self.get_reg_view(H, L);
            self.write(loc, A);
            self.l -= 1;
        }
        else if opcode == 0x2A {
            let loc = self.get_reg_view(H, L);
            let n = self.read(loc, gamepack);
            self.set_reg(A, n);
            self.l += 1;
        }
        else if opcode == 0x22 {
            let loc = self.get_reg_view(H, L);
            self.write(loc, A);
            self.l += 1;
        }
        
        
        else if opcode >> 6 == LDREG {
            println!("load 8-bit register");
            let x = (opcode >> XOFFSET) & REGMASK;
            let y = (opcode >> YOFFSET) & REGMASK;
            let mut value: u8 = self.get_reg(y);

            if y == 0b110 {
                let loc = self.get_reg_view(H, L);
                value = self.read(loc, gamepack);
            }
            if x == 0b110 {
                let loc = self.get_reg_view(H, L);
                self.write(loc, value);
                ()
            }
            self.set_reg(x, value);    
        }
        else if (opcode >> 5) == 0 && opcode & 0b111 == LDIMM {
            println!("load imm");
            let x = (opcode >> XOFFSET) & REGMASK;
            let y = (opcode >> YOFFSET) & REGMASK;
            let n = self.fetch();

            if x == 0b110 && y == 0b110 {
                let loc = self.get_reg_view(H, L);
                self.write(loc, n);
                ()
            }
            else{
                self.set_reg(x, n)
            }
        }

        // 16-bit load / store instructions
        // LDRRNNIMM    = 0b00xx0001
        // LDSPNN       = 0b00001000
        
        //load reg view with immediate
        else if opcode & 0x0001 != 0 && opcode >> 6 == 0 {
            let rr_key = opcode >> 4;
            let lsb = self.fetch();
            let msb = self.fetch();
            let nn = u8_to_u16(lsb, msb); 
            self.load_rr(rr_key, nn);
        }

        // write to memory from sp
        else if opcode == 0x08 {
            let lsb = self.fetch();
            let msb = self.fetch();
            let loc = u8_to_u16(lsb, msb);
            self.write(loc, high(self.sp));
            self.write(loc + 1, low(self.sp));
        }

        // load stack pointer from HL
        else if opcode == 0xF9 {
            self.load_rr(SP, self.get_reg_view(H, L))
        }
            
        else if opcode >> 6 == 0b11 {
            let rr_key = high_u8(opcode - 0b11000000);

            //push to stack 0b11xx0101
            if opcode & 0b0101 == 0b0101 {
                let rr = self.get_rr(rr_key);
                let lsb = high(rr);
                let msb = low(rr);
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

        }else{
            println!("did nothing lol");
        }
    }

    fn set_rr_from_u16(&mut self, r1: u8, r2: u8, nn: u16) {
        self.set_reg(r1, (nn >> 8) as u8);
        self.set_reg(r2, nn as u8);
    }

    fn get_rr(&self, rr_key: u8) -> u16 {
        match rr_key {
            BC => self.get_reg_view(B, C),
            DE => self.get_reg_view(D, E),
            HL => self.get_reg_view(H, L),
            SP => self.sp,
            _ => 0x11
        }
    }

    fn load_rr(&mut self, rr_key: u8, nn: u16) {
        match rr_key {
            BC => self.set_rr_from_u16(B, C, nn),
            DE => self.set_rr_from_u16(D, E, nn),
            HL => self.set_rr_from_u16(H, L, nn),
            SP => self.sp = nn,
            _ => ()
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
            _ => (),
        }
    }

    fn get_reg(&self, reg: u8) -> u8{
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,
            _ => 0
        }
    }

    fn set_reg_e(&mut self, x: u8, n: u8) -> Result<(), InvalidRegError> {
        match x {
            0x0 => self.b = n,
            0x1 => self.c = n,
            0x2 => self.d = n,
            0x3 => self.e = n,
            0x4 => self.h = n,
            0x5 => self.l = n,
            0x7 => self.a = n,
            _ => return Err(InvalidRegError),
        }

        Ok(())

    }

    fn get_reg_e(&self, reg: u8) -> Result<u8, InvalidRegError> {
        match reg {
            0x0 => Ok(self.b),
            0x1 => Ok(self.c),
            0x2 => Ok(self.d),
            0x3 => Ok(self.e),
            0x4 => Ok(self.h),
            0x5 => Ok(self.l),
            0x7 => Ok(self.a),
            _ => Err(InvalidRegError)
        }
    }

    fn read(&self, addr: u16, gamepack: &Memory) -> u8{
        gamepack.read(addr)
    }

    fn write(&mut self, addr: u16, reg: u8){
        println!("wrote {0:02X} at address {1:#018b}", reg, addr);
        self.mem_write_stack.push((reg, addr));
        self.mem_write += 1;
    }

    pub fn print_info(&self){
        println!("{:#?}", self);
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
