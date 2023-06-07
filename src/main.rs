pub mod cpu;
pub mod instructionset;

use crate::cpu::*;

pub const KBIT: i32 = 1024;
pub const KBYTE: i32 = 8 * KBIT;

#[derive(Debug)]
struct GameBoy {
    dmgcpu: DMGCPU,
    ram: Memory,
    vram: Memory,
}

impl GameBoy {
    fn new() -> GameBoy {
        GameBoy {
            dmgcpu: DMGCPU::new(),
            ram: Memory::RAM { size: 8 * KBYTE },
            vram: Memory::RAM { size: 8 * KBYTE },
        }
    }
    
    fn read_instr(&mut self) {
        for _ in 0..8 {
           self.dmgcpu.decode();
           self.print_info();
        }
    }

    fn print_info(&self) {
        self.dmgcpu.print_info();
        //println!("{:#?}", self);
    }
}

fn main() {
    let mut my_gb = GameBoy::new();
    my_gb.print_info();

    my_gb.read_instr();

}
