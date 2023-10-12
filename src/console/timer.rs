use std::time::{Instant, Duration};

use crate::console::*;

const DIV_RATE: u32 = 16384;
const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;

const DIV_DUR: Duration = Duration::new(0, 1_000_000_000u32 / DIV_RATE);

pub struct HTimer {
    div: u8,

}

impl HTimer {
    
    pub fn new() -> HTimer {
        HTimer {
            div: 0,
        }
    }

    //maybe return sterrupts
    pub fn update_div(&mut self,  start: Instant){
        if start.elapsed() > DIV_DUR {
            match self.div.overflowing_add(1) {
                (value, false) => self.div = value,
                (_, true) => self.div = 0
            }
            println!("div: {}", self.div);
        }
                
    }
}


