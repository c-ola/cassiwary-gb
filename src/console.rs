pub mod cpu;
pub mod memory;

use crate::cpu::*;
use crate::memory::*;

#[derive(Debug)]
pub struct GameBoy {
    dmgcpu: DMGCPU,
    ram: Memory,
    vram: Memory,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            dmgcpu: DMGCPU::new(),
            ram: Memory::RAM { size: 8 * KBYTE },
            vram: Memory::RAM { size: 8 * KBYTE },
        }
    }
    
    pub fn read_instr(&mut self) {
        for _ in 0..10 {
           self.dmgcpu.decode();
           self.print_info();
        }
    }

    pub fn print_info(&self) {

        self.dmgcpu.print_info();
        println!("");
        //println!("{:#?}", self);
    }
}
