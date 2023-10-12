use std::fs::File;
use std::io::Write;
use std::fs::read;
use std::io::Result;

pub const KBIT: usize = 1024;
pub const KBYTE: usize = 8 * KBIT;

pub const GLOBAL_START: u16 = 0x30;
pub const BOOT_ROM_PATH: &str = "./gb_boot/DMG_ROM.gb";

// size in bits
#[derive(Debug)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {

    pub fn new(size: usize) -> Memory {
        Memory {
            data: vec![0x00; size],
        }
    }
    
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn quick_init(&mut self){
        self.write(0xFF00, 0xCF);
        self.write(0xFF01, 0x00);
        self.write(0xFF02, 0x7E);
        self.write(0xFF04, 0x18);
        self.write(0xFF05, 0x00);
        self.write(0xFF06, 0x00);
        self.write(0xFF07, 0xF8);
        self.write(0xFF0F, 0xE1);
        self.write(0xFF10, 0x80);
        self.write(0xFF11, 0xBF);
        self.write(0xFF12, 0xF3);
        self.write(0xFF13, 0xFF);
        self.write(0xFF14, 0xBF);
        self.write(0xFF16, 0x3F);
        self.write(0xFF17, 0x00);
        self.write(0xFF18, 0xFF);
        self.write(0xFF19, 0xBF);
        self.write(0xFF1A, 0x7F);
        self.write(0xFF1B, 0xFF);
        self.write(0xFF1C, 0x9F);
        self.write(0xFF1D, 0xFF);
        self.write(0xFF1E, 0xBF);
        self.write(0xFF20, 0xFF);
        self.write(0xFF21, 0x00);
        self.write(0xFF22, 0x00);
        self.write(0xFF23, 0xBF);
        self.write(0xFF24, 0x77);
        self.write(0xFF25, 0xF3);
        self.write(0xFF26, 0xF1);
        self.write(0xFF40, 0x91);
        self.write(0xFF41, 0x81);
        self.write(0xFF42, 0x00);
        self.write(0xFF43, 0x00);
        self.write(0xFF44, 0x91);
        self.write(0xFF45, 0x00);
        self.write(0xFF46, 0xFF);
        self.write(0xFF47, 0xFC);
        //self.write(0xFF48, 0x00);
        //self.write(0xFF49, 0x00);
        self.write(0xFF4A, 0xFF);
        self.write(0xFF4B, 0xFF);
        self.write(0xFF4D, 0xFF);
        self.write(0xFF4F, 0xFF);
        self.write(0xFF51, 0xFF);
        self.write(0xFF52, 0xFF);
        self.write(0xFF53, 0xFF);
        self.write(0xFF54, 0xFF);
        self.write(0xFF55, 0xFF);
        self.write(0xFF55, 0xFF);
        self.write(0xFF58, 0xFF);
        self.write(0xFF69, 0xFF);
        self.write(0xFF6A, 0xFF);
        self.write(0xFF6B, 0xFF);
        self.write(0xFF70, 0xFF);
        self.write(0xFFFF, 0x00);
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, n: u8){
        self.data[addr as usize] = n;
    }

    pub fn log(&self) -> Result<()> {
        let mut f = File::create("memory.hex")?;
        println!("Creating hex dump");
        for i in (0..self.data.len()).step_by(16){
            write!(f, "{:04X}:", i)?;
            for j in 0..16 {
                write!(f, " {:02X}", self.data[i + j])?;
            }
            writeln!(f, "")?;
        }

        Ok(())
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

