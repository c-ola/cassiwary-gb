use std::fs::File;
use std::io::{Write, Result};

pub const KBIT: usize = 1024;
pub const KBYTE: usize = 8 * KBIT;

pub const GLOBAL_START: u16 = 0x30;
pub const BOOT_ROM_PATH: &str = "./gb_boot/DMG_ROM.gb";

const DMA: u16 = 0xFF46;
const JOYP: u16 = 0xFF00;

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

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, byte: u8){
        if addr >= 0xC000 && addr <= 0xDDFF {
            self.data[(addr + 0x2000) as usize] = byte;
        }

        match addr {
            DMA => self.dma_transfer(byte),
            JOYP => {
                //println!("{:#10b}", self.read(0xFF00));
                self.data[addr as usize] = (byte & 0xF0) + (self.data[addr as usize] & 0x0F);
            },
            _ => self.data[addr as usize] = byte,

        }
    }

    pub fn write_io(&mut self, addr: u16, byte:u8) {
        self.data[addr as usize] = byte;
    }

    fn dma_transfer(&mut self, source: u8) {
        let length = 0x9F;
        let source = (source as u16) << 8;
        let dest = 0xFE00;
        
        for i in 0..length {
            let byte = self.read(source + i);
            self.write(dest + i, byte);
        }
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

    pub fn print(&self, addr: u16, lines: u16){

        println!("Showing address from Memory: {0:04X} to {1:04X}", addr, addr + lines * 16);

        for i in (addr..(addr + lines * 16)).step_by(16) {
            print!("{:04X}:", i);
            std::io::stdout().flush().unwrap();
            for j in 0..16 as usize {
                let index = i + j as u16;
                print!(" {:02X}", self.data[index as usize]);
                std::io::stdout().flush().unwrap();
            }
            println!("");
        }
    }
}

