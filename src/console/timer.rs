use interrupts::TIMER_I;

use crate::{console::*, test_bit};

const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;

//const CLOCK_SPEEDS: [usize; 4] = [1024, 16, 64, 256];
const CLOCK_SPEEDS: [usize; 4] = [256, 4, 16, 64];
const DIV_SPEED: usize = 256;

pub struct HTimer {
    div_dots: usize,
    tima_dots: usize,
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl HTimer {
    pub fn new() -> HTimer {
        HTimer {
            div_dots: 0usize,
            tima_dots: 0usize,
            div: 0,
            tima: 0u8,
            tma: 0u8,
            tac: 0xf8u8,
        }
    }

    pub fn get_registers(&mut self, memory: &Memory) {
        self.div = memory.read(DIV);
        self.tima = memory.read(TIMA);
        self.tma = memory.read(TMA);
        self.tac = memory.read(TAC);
    }

    pub fn set_registers(&mut self, memory: &mut Memory) {
        memory.write(DIV, self.div);
        memory.write(TIMA, self.tima);
        memory.write(TMA, self.tma);
        memory.write(TAC, self.tac);
    }

    pub fn update(&mut self, stopped: bool, memory: &mut Memory) {
        self.get_registers(memory);
        //println!("{}, {}, {}, {}", self.div, self.tima, self.tma, self.tac);
        if stopped {
            self.div = 0;
        }
        if self.div_dots >= DIV_SPEED && !stopped {
            self.div = self.div.overflowing_add(1).0;
            self.div_dots = 0;
        }

        let clk_s = self.tac & 0b11;
        if self.tima_dots >= CLOCK_SPEEDS[clk_s as usize] {
            let tima_en = test_bit!(self.tac, 2);
            if tima_en {
                let sum = self.tima.overflowing_add(1);
                if sum.1 {
                    self.tima = self.tma;
                    memory.request_interrupt(TIMER_I);
                } else {
                    self.tima = sum.0;
                }
            }
            self.tima_dots = 0;
        }

        self.div_dots += 1;
        self.tima_dots += 1;

        self.set_registers(memory);
    }
}
