const JOYP: u16 = 0xFF00;

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

use super::memory::{self, Memory};

#[derive(Default)]
pub struct Joypad {
    input: u8,
    keys: HashSet<Keycode>,
}

impl Joypad {

    pub fn update(&mut self, memory: &mut Memory) {
        self.input = memory.read(JOYP);
        if self.keys.contains(&Keycode::X) || self.keys.contains(&Keycode::Right) {
            self.input |= 0b0001;
        } 
        if self.keys.contains(&Keycode::Z) || self.keys.contains(&Keycode::Left) {
            self.input |= 0b0010;
        } 
        if self.keys.contains(&Keycode::Return) || self.keys.contains(&Keycode::Up){
            self.input |= 0b0100;
        } 
        if self.keys.contains(&Keycode::Tab) || self.keys.contains(&Keycode::Down){
            self.input |= 0b1000;
        } 

        memory.write(JOYP, self.input);

    }
    
}
