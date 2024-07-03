use std::cmp::min;
use std::fs::File;
use std::fs::read;
use std::io::{Write, Result};

pub const KBIT: usize = 1024;
pub const KBYTE: usize = 8 * KBIT;
pub const BYTE: usize = 8;

pub const GLOBAL_START: u16 = 0x30;
pub const BOOT_ROM_PATH: &str = "./gb_boot/DMG_ROM.gb";

const DMA: u16 = 0xFF46;
const JOYP: u16 = 0xFF00;
use super::regids::IF;

// size in bits
#[derive(Debug, Clone)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {

    pub fn new(size: usize) -> Memory {
        let mut data = vec![0x00; size];
        if size > 0xFF00 {
            data[0xFF00] = 0b00110000;
        }
        Memory {
            data,
        }
    }

    pub fn from_file(size: usize, rom_path: &str) -> Memory {
        let mut data = vec![0x00; size];
        match read(rom_path) {
            Ok(buffer) => {
                //load default boot rom
                for i in 0..min(buffer.len(), size) {
                    data[i] = buffer[i];
                }
            },
            Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
        }

        Memory {
            data
        }

    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn read_io(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
    pub fn read(&self, addr: u16) -> u8 {
        if  addr == 0xFF02 {
            //println!("reading serial ");
            //println!("readd joyp, {:#010b}", self.data[addr as usize]);
            //self.data[addr as usize] = (byte & 0xF0) + (self.data[addr as usize] & 0x0F);
        }
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, byte: u8){
        if  addr == 0xFF02 {
            //println!("writing serial ctrl {:#010b}", byte);
        }
        if  addr == 0xFF01 {
            //println!("writing serial data {:#010b}", byte);
        }
        if addr >= 0xC000 && addr <= 0xDDFF {
            self.data[(addr) as usize] = byte;
            self.data[(addr + 0x2000) as usize] = byte;
        }
        if addr >= 0xE000 && addr <= 0xFDFF {
            self.data[(addr - 0x2000) as usize] = byte;
            self.data[(addr) as usize] = byte;

        }
        match addr {
            0xFF42 => {
                self.data[addr as usize] |= byte & 0b1111_1100;
            }
            DMA => {
                self.dma_transfer(byte);
                self.data[addr as usize] = byte
            }
            JOYP => {
                //println!("writing normal {:#010b}, {:#010b}", byte, self.data[addr as usize]);
                self.data[addr as usize] = (byte & 0xF0) + (self.data[addr as usize] & 0x0F);
            },
            _ => self.data[addr as usize] = byte,
        }
    }

    pub fn write_io(&mut self, addr: u16, byte:u8) {
        //println!("writing io {:#010b}, {:#010b}", byte, self.data[addr as usize]);
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

    pub fn request_interrupt(&mut self, interrupt: u8) {
        let if_old = self.read(IF);
        let if_new = if_old | interrupt;
        self.write(IF, if_new);
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

    pub fn print(&self, addr: u16, end_addr: u16){
        let end_addr = if end_addr < addr {
            addr
        } else {
            end_addr
        };
        println!("Showing address from Memory: {0:04X} to {1:04X}", addr, end_addr);

        for i in (addr..end_addr).step_by(16) {
            print!("{:04X}:", i);
            std::io::stdout().flush().unwrap();
            for j in 0..16 as usize {
                let index = i + j as u16;
                if index > end_addr {
                    println!("");
                    break;
                }
                print!(" {:02X}", self.data[index as usize]);
                std::io::stdout().flush().unwrap();
            }
            println!("");
        }
    }
}

