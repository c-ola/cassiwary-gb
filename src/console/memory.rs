use std::fs::File;
use std::io::Write;
use std::fs::read;
use std::io::Result;

pub const KBIT: usize = 1024;
pub const KBYTE: usize = 8 * KBIT;

pub const GLOBAL_START: u16 = 0x30;

// size in bits
#[derive(Debug)]
pub struct Memory {
    data: Vec<u8>,
}

const INSTR_SIZE: usize = 22;
const USE_SCUFFED_INSTRUCTINOS: bool = false;
const SCUFFED_INSTRUCTIONS: [u8; INSTR_SIZE] = [
    0b0011_1110, //load A, 0x05
    0b0000_0101,

    0b0000_0110, //load B, 0x03
    0b0000_0011,

    0b1000_0000, //add B

    0b0010_0001, //load HL, nn
    0b0000_0000, // nn = 0002
    0b0000_0010,

    0b0011_0110, // load (HL), n
    0b1110_0011, // n = 0xE3

    0b1110_0101, // push HL

    0b1101_0001, // pop DE

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
    ];

impl Memory {

    pub fn init_memory(&mut self){
        
        let filename = "roms/DMG_ROM.bin";

        let buffer = match USE_SCUFFED_INSTRUCTINOS {
            true => SCUFFED_INSTRUCTIONS.to_vec(),
            false => {
                match read(filename) {
                    Ok(result) => result,
                    Err(error) => {
                        println!("{error}");
                        SCUFFED_INSTRUCTIONS.to_vec()
                    },
                }
            }
        };

        for i in 0..buffer.len() {
            self.write(i as u16, buffer[i]);
        }

        let boot_rom = "roms/DMG_ROM.bin";
        
        match read(boot_rom) {
            Ok(result) => {
                for i in 0..0x0100 {
                    //self.write(i, result[i as usize]);
                }
            },
            Err(error) => panic!("boot rom error, file not found or incorrect file"),
        }


        //self.setup_boot();
    }

    pub fn log(&self) -> Result<()> {
        let mut f = File::create("memory.hex")?;

        for i in (0..self.data.len()).step_by(16){
            write!(f, "{:04X}:", i)?;
            for j in 0..16 {
                write!(f, " {:02X}", self.data[i + j])?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }

    pub fn new(size: usize) -> Memory {
        Memory {
            data: vec![0; size],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, n: u8){
        self.data[addr as usize] = n;
    }

    pub fn print(&self, addr: usize, lines: usize){

        println!("Showing address from Memory: {0:04X} to {1:04X}", addr, addr + lines * 16);

        for i in (addr..(addr + lines * 16)).step_by(16) {
            print!("{:04X}:", i);
            std::io::stdout().flush().unwrap();
            for j in 0..16 {
                print!(" {:02X}", self.data[i + j]);
                std::io::stdout().flush().unwrap();
            }
            println!("");
        }
    }
}

