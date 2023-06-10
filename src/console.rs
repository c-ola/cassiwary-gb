pub mod cpu;
pub mod memory;

use crate::cpu::*;
use crate::memory::*;

pub const B: u8 = 0x0;
pub const C: u8 = 0x1;
pub const D: u8 = 0x2;
pub const E: u8 = 0x3;
pub const H: u8 = 0x4;
pub const L: u8 = 0x5;
pub const A: u8 = 0x7;
pub const READ_HL: u8 = 0x6;
pub const BC: u8 = 0x0;
pub const DE: u8 = 0x1;
pub const HL: u8 = 0x2;
pub const SP: u8 = 0x3;


#[derive(Debug)]
pub struct GameBoy {
    run_on_boot: bool,
    instruction_count: u16,
    accumulator: u32,
    cpu: SharpSM83,
    pub gamepack: Memory,
    pub write_idx: u16,
}

impl GameBoy {
    pub fn peek_cpu(&self) -> &SharpSM83 {
        &self.cpu
    }

    pub fn peek_memory(&self) -> &Memory {
        &self.gamepack
    }

    pub fn new() -> GameBoy {
        GameBoy {
            run_on_boot: true,
            accumulator: 0,
            instruction_count: 50,
            cpu: SharpSM83::new(),
            gamepack: Memory::new(8 * KBYTE),
            write_idx: 0,

        }
    }
    
    pub fn set_boot(&mut self, boot: bool) {
        self.run_on_boot = boot;
    }

    pub fn set_run_count(&mut self, count: u16) {
        self.instruction_count = count;
    }
    

    pub fn init(&mut self) {

        println!("Initial Memory");
        self.gamepack.init_memory();
        self.gamepack.print(0, 5);

    }
    
    pub fn run_emulator(&mut self) {
        println!("---------BEGINNING EMULATION---------");

        self.cpu.init_emu();

        while !self.cpu.stop && self.accumulator < 10000 {
            
            match self.cpu.run(&self.gamepack) {
                Ok(()) => {
                    while self.cpu.mem_write_stack.len() > 0 {
                        self.cpu.mem_write -= 1;
                        let dat = self.cpu.mem_write_stack.pop();

                        match dat {
                            Some((x, y)) => self.gamepack.write(y, x),
                            _ => ()
                        }
                    }
                },
                Err(error) => println!("{error}"),
            }

            self.accumulator += 1;
        }

        self.log_memory();

    }

    pub fn start(&mut self) {

        if !self.run_on_boot {
            println!("not running");
            return
        }

        println!("Reading instructinos");

        while self.accumulator < self.instruction_count as u32 {
            println!("Instruction: {0}", self.accumulator);
            match self.cpu.run(&self.gamepack) {
                Ok(()) => {
                    while self.cpu.mem_write_stack.len() > 0 {
                        self.cpu.mem_write -= 1;
                        let dat = self.cpu.mem_write_stack.pop();

                        match dat {
                            Some((x, y)) => self.gamepack.write(y, x),
                            _ => ()
                        }
                    }
                },
                Err(error) => println!("{error}"),
            }

            self.accumulator += 1;
        }

    }

    pub fn inc_instr_count(&mut self){
        self.instruction_count += 1;
    }

    pub fn log_memory(&self) {
        match self.gamepack.log() {
            Ok(()) => (),
            Err(error) => println!("{error}"),
        }
    }

    pub fn print_info(&self) {
    }

    pub fn write(&mut self, data: u8) {
        self.gamepack.write(self.write_idx, data);
        self.write_idx += 1;
    }
}
