use std::fmt;
use std::error::Error;

// sharp sm83 @ 8.4 or 4.2 mhz
// 256 B rom
// get this thing working first (it was the gameboys cpu)
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
            .field("f", &format_args!("{:#010b}", self.f))
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

const SCUFFED_INSTRUCTIONS: [u8; 12] = 
[
    0b0010_1110,
    0b0011_1100, // ldimm
    0b0100_0101, // ldreg
    0b0100_1110, // ld hl
    0b0111_0000, // str hl
    0b0011_0110,
    0b0010_0110, // str hl imm
    0b0000_0000, // str hl
    0b0000_0000,
    0b0000_0000, // str hl
    0b0000_0000,
    0b0000_0000, // str hl

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
    pc: usize,
    sp: u16,

}

const B: u8 = 0x0;
const C: u8 = 0x1;
const D: u8 = 0x2;
const E: u8 = 0x3;
const H: u8 = 0x4;
const L: u8 = 0x5;
const A: u8 = 0x7;
const SP_KEY: u8 = 0x3;

struct RR {
    key: u8,
    rh: u8,
    rl: u8,
}

const BC: RR = RR {
    key: 0x0,
    rh: B,
    rl: C,
};

const DE: RR = RR {
    key: 0x0,
    rh: D,
    rl: E,
};

const HL: RR = RR {
    key: 0x0,
    rh: H,
    rl: L,
};

const SP: RR = RR {
    key: 0x0,
    rh: 0x3,
    rl: 0x3,
};



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

    fn fetch(&mut self) -> u8 {

        // ---- get instruction from memory  ----
        let opcode = SCUFFED_INSTRUCTIONS[self.pc];
        println!("fetched instruction: {:#08b} at pc: {1}", opcode, self.pc);
        self.pc += 1;

        opcode
    }
    

    // REDO SO THAT EACH INSTRUCTION IS 8 BITS AND THE NEXT Immediate/etc valueS CAN BE FOUND FROM
    // INCREMENTING THE PC
    fn decode(&mut self){

        let opcode = self.fetch();

        if opcode == 0 {
            ()
        }
        
        // refactor this to clean it up maybe instead of 100 if statements
        // 8-bit load / store instructions
        if opcode == 0x0A {
            let loc = self.get_reg_view(B, C); 
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
        } 
        if opcode == 0x1A {
            let loc = self.get_reg_view(D, E); 
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
        }
        if opcode == 0x02 {
            let loc = self.get_reg_view(B, C); 
            self.write(loc, A);
        }
        if opcode == 0x12 {
            let loc = self.get_reg_view(D, E); 
            self.write(loc, A);
        }
        if opcode == 0xFA {
            let lsb = self.fetch();
            let msb = self.fetch();
            let loc = self.u8_to_u16(lsb, msb);
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
        } 
        if opcode == 0xEA {
            let lsb = self.fetch();
            let msb = self.fetch();
            let loc = self.u8_to_u16(lsb, msb);
            self.write(loc, A);
        } 
        if opcode == 0xF2 {
            let loc = self.u8_to_u16(0xFF, self.get_reg(C));
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
        }
        if opcode == 0xE2 {
            let loc = self.u8_to_u16(0xFF, self.get_reg(C));
            self.write(loc, A)
        }
        if opcode == 0xF0 {
            let lsb = self.fetch(); 
            let loc = self.u8_to_u16(0xFF, lsb);
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
        }
        if opcode == 0xF0 {
            let lsb = self.fetch(); 
            let loc = self.u8_to_u16(0xFF, lsb);
            self.write(loc, A);
        }
        if opcode == 0x3A {
            let loc = self.get_reg_view(H, L);
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
            self.l -= 1;
        }
        if opcode == 0x32 {
            let loc = self.get_reg_view(H, L);
            self.write(loc, A);
            self.l -= 1;
        }
        if opcode == 0x2A {
            let loc = self.get_reg_view(H, L);
            let n = self.get_memory_at_addr(loc);
            self.set_reg(A, n);
            self.l += 1;
        }
        if opcode == 0x22 {
            let loc = self.get_reg_view(H, L);
            self.write(loc, A);
            self.l += 1;
        }
        
        
        if opcode & LDREG != 0 {

            let x = (opcode >> XOFFSET) & REGMASK;
            let y = (opcode >> YOFFSET) & REGMASK;
            let mut value: u8 = self.get_reg(y);

            if y == 0b110 {
                let loc = self.get_reg_view(H, L);
                value = self.get_memory_at_addr(loc);
            }
            if x == 0b110 {
                let loc = self.get_reg_view(H, L);
                self.write(loc, value);
                ()
            }
            self.set_reg(x, value);    

        }
        else if opcode & LDIMM != 0 {
            let x = (opcode >> XOFFSET) & REGMASK;
            let y = (opcode >> YOFFSET) & REGMASK;
            let n = self.fetch();

            if x == 0b110 && y == 0b110 {
                let loc = self.get_reg_view(H, L);
                self.write(loc, n);
                ()
            }
            else if y == 0b010 {
                
            }
            else{
                self.set_reg(x, n)
            }
        }

        // 16-bit load / store instructions
        // LDRRNNIMM    = 0b00xx0001
        // LDSPNN       = 0b00001000
        
        //load reg view with immediate
        if opcode & 0x1 != 0 {
            let reg = self.get_reg_view_rr(opcode >> 4);

        }



    }
    
    fn load_view(&self, rr: u8) {
        
    }

    fn get_reg_view_rr(&self, rr: u8) -> u16 {
        0x1
    }

    fn get_reg_view(&self, x: u8, y: u8) -> u16 {
        self.u8_to_u16(self.get_reg(x), self.get_reg(y))
    }
    
    fn u8_to_u16(&self, high: u8, low: u8) -> u16 {
        (high as u16) * (2 << 8) + low as u16
    }

    fn set_reg(&mut self, x: u8, n: u8) {
        match x {
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

#[derive(Debug)]
struct InvalidRegError;

impl Error for InvalidRegError {}

impl fmt::Display for InvalidRegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid specified register")
    }
}



// ARM7TDMI @ 16.78MHz\
// the new one for the game boy advanced
struct AGB {

}
