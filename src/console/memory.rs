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

impl Memory {

    pub fn new(size: usize) -> Memory {
        Memory {
            data: vec![0; size],
        }
    }

    pub fn init_memory(&mut self, rom_path: Option<&str>){

        match rom_path {
            Some(game_rom) => {
                match read(game_rom) {
                    Ok(buffer) => {
                        for i in 0..buffer.len() {
                            self.write(i as u16, buffer[i]);
                        }
                    }
                    Err(error) => panic!("{error} no game rom was specified or found"),
                }

            },
            None => println!("No game rom was found: running boot rom only"),
        };
        let boot_rom = "gb_boot/DMG_ROM.bin";

       match read(boot_rom) {
            Ok(result) => {
                for i in 0..0x0100 {
                    self.write(i, result[i as usize]);
                }
            },
            Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
        }


    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, n: u8){
        self.data[addr as usize] = n;
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

