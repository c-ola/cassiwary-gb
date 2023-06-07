// sharp sm83 @ 8.4 or 4.2 mhz
// 256 B rom
// get this thing working first (it was the gameboys cpu)

use crate::instructionset::*;
use std::fmt;

#[derive(Debug)]
pub struct DMGCPU {
    cpu: SharpSM83,
    //ROM_256B: ROM,

    //APU: AudioProcessingUnit,
    //PPU: PictureProcessingUnit,
}

impl DMGCPU {
    pub fn new() -> DMGCPU {
        DMGCPU {
            cpu: SharpSM83::new(),
        }
    }

    pub fn decode(&mut self){
        self.cpu.decode();
    }

    pub fn print_info(&self){
        self.cpu.print_info();
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
            .field("f", &format_args!("{:#08b}", self.f))
            .finish()
    }
}

const DMG: f32 = 4.194304;
const CGB: f32 = 8.388608;

const LDREG: u8 = 0b01000000;
const LDIMM: u8 = 0b110;

const XOFFSET: u8 = 3;
const YOFFSET: u8 = 0;
const REGMASK: u8 = 0b00000111;

const SCUFFED_INSTRUCTIONS: [u16; 8] = 
[
    0b0010_1110_0011_1100, // ldimm
    0b0100_0101_0000_0000, // ldreg
    0b0100_1110_0000_0000, // ld hl
    0b0111_0000_0000_0000, // str hl
    0b0011_0110_0010_0110, // str hl imm
    0b0000_0000_0000_0000, // str hl
    0b0000_0000_0000_0000, // str hl
    0b0000_0000_0000_0000, // str hl

];

// the cpu
//#[derive(Debug)]
pub struct SharpSM83 {
    // 8-bit general purpose not even needed???? just split up 16 bit maybe
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    //8-bit flag / 0-3 grounded to 0, 4 carry flag C, 5, half-carry H, 6 negative N, 7 zero Z
    f: u8,

    // 16-bit general purpose register views
    //af: u16, //cant write 0-3
    //bc: u16,
    //de: u16,
    //hl: u16,

    //16-bit special purpose
    pc: u16,
    sp: u16,

}

impl SharpSM83 {
    
    pub fn new() -> SharpSM83 {
        SharpSM83 {
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
            sp: 0x00000000,
        }

    }  

    fn fetch(&mut self) -> u16 {

        // ---- get instruction from memory  ----
        let opcode = SCUFFED_INSTRUCTIONS[self.pc as usize];
        println!("{:#016b}", opcode);
        self.pc += 1;

        opcode
    }

    fn decode(&mut self){
        let opcode = self.fetch();

        if opcode == 0 {
            ()
        }

        let bytes = opcode.to_be_bytes(); 
        let high = bytes[0];
        let low = bytes[1];
        
        

        if high & LDREG != 0 {
            let x = (high >> XOFFSET) & REGMASK;
            let y = (high >> YOFFSET) & REGMASK;
            let reg_value = self.get_reg(y);
            let mut ld_value = reg_value;

            if y == 0b110 {
                let loc = (self.h as u16) * (2 << 8) + self.l as u16;
                ld_value = self.get_memory_at_addr(loc);
            }
            if x == 0b110 {
                let loc = (self.h as u16) * (2 << 8) + self.l as u16;
                self.write(loc, self.get_reg(y));
                ()
            }


            self.set_reg(x, ld_value);

        }
        else if high & LDIMM != 0 {
            let x = (high >> XOFFSET & REGMASK);
            let y = (high >> YOFFSET) & REGMASK;
            let n = low;

            if x == 0b110 && y == 0b110 {
                let loc = (self.h as u16) * (2 << 8) + self.l as u16;
                self.write(loc, n);
                ()
            }
            self.set_reg(x, n);
        }
    }

    fn set_reg(&mut self, x: u8, n: u8){
        let value = n;
        match x {
            0x0 => self.b = value,
            0x1 => self.c = value,
            0x2 => self.d = value,
            0x3 => self.e = value,
            0x4 => self.h = value,
            0x5 => self.l = value,
            0x7 => self.a = value,
            _ => ()
        }

    }

    fn get_reg(&self, reg: u8) -> u8 {
        match reg {
            0x0 => self.b,
            0x1 => self.c,
            0x2 => self.d,
            0x3 => self.e,
            0x4 => self.h,
            0x5 => self.l,
            0x7 => self.a,
            _ => 0
        }
    }

    fn get_memory_at_addr(&self, addr: u16) -> u8{
        // memory[addr]
        0b10011001
    }

    fn write(&self, addr: u16, reg: u8){
        println!("wrote {0:02X} at address {1:#016b}", reg, addr)
    }

    fn print_info(&self){
        println!("{:#?}", self);
    }

}
struct ROM {
    size: i32,
}


// size in bits
#[derive(Debug)]
pub enum Memory {
    RAM {size: i32},
    VRAM {size: i32},
    ROM {size: i32}
}

impl Memory {
    //data should be an array i think?
    fn write(&self, loc: i32, size: i32, data: i32){

    }
}

// ARM7TDMI @ 16.78MHz\
// the new one for the game boy advanced
struct AGB {

}
