use crate::console::*;

const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;

const CLOCK_SPEEDS: [usize; 4] = [1024, 16, 64, 256]; 
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
            tac: 0u8,
        }
    }

    fn get_registers(&mut self, memory: &Memory) {
        self.div = memory.read(DIV);
        self.tima = memory.read(TIMA);
        self.tma = memory.read(TMA);
        self.tac = memory.read(TAC);
    }

    fn set_registers(&mut self, memory: &mut Memory) {
        memory.write(DIV, self.div);
        memory.write(TIMA, self.tima);
        memory.write(TMA, self.tma);
        memory.write(TAC, self.tac);
    }

    pub fn update(&mut self, memory: &mut Memory){
        self.get_registers(memory);

        
        if self.div_dots > DIV_SPEED {
            self.div();
            self.div_dots = 0;
        }         

        let clk_s = self.tac & 0b11;
        if self.tima_dots > CLOCK_SPEEDS[clk_s as usize] {
            if self.tima() {
                HTimer::request_interrupt(2, memory);
            }
            self.tima_dots = 0;
        }   
        
        self.div_dots += 1;
        self.tima_dots += 1;
        
        self.set_registers(memory);
    }

    fn div(&mut self) {
        self.div = self.div.overflowing_add(1).0;
    }

    fn tima(&mut self) -> bool {
        let tima_en = (self.tac & 0b100) != 0;

        if tima_en {
            match self.tima.overflowing_add(1) {
                (value, false) => self.tima = value,
                (_, true) => self.tima = self.tma,
            }
            true
        }else {false}
    }

    fn request_interrupt(bit: u8, memory: &mut Memory) {
        let if_old = memory.read(IF);
        let if_new = if_old | (0b1 << bit);
        memory.write(IF, if_new);
    }
}

