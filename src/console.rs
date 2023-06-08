pub mod cpu;
pub mod memory;

use crate::cpu::*;
use crate::memory::*;

#[derive(Debug)]
pub struct GameBoy {
    cpu: SharpSM83,
    gamepack: Memory,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            cpu: SharpSM83::new(),
            gamepack: Memory::new(KBYTE),
        }
    }
    
    pub fn read_instr(&mut self) {
        self.gamepack.print(0, 5);
        self.gamepack.write(0, 0b10010101);

        let count = 7usize;

        while self.cpu.pc < count {

            self.cpu.run(&self.gamepack);

            while self.cpu.mem_write != 0 {
                self.cpu.mem_write -= 1;
                let dat = self.cpu.mem_write_stack[self.cpu.mem_write as usize];
                self.gamepack.write(dat.1, dat.0);
            }
            self.gamepack.print(0, 5);

        }
        self.gamepack.print(0, 5);
    }

    pub fn print_info(&self) {
        self.cpu.print_info();
        println!("");
        //println!("{:#?}", self);
    }
}
