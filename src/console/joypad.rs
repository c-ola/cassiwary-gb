const JOYP: u16 = 0xFF00;

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

use super::memory::Memory;

#[derive(Default, Debug)]
pub struct Joypad {
    start: bool,
    select: bool,
    b: bool,
    a: bool,
    down: bool,
    up: bool,
    left: bool,
    right: bool,
    
    input: u8,
    buttons: u8,
    dpad: u8,
}

impl Joypad {

    pub fn update(&mut self, memory: &mut Memory, keys: &HashSet<Keycode>) {
        let sel_buttons = memory.read(JOYP) & 0b0001_0000 == 0;
        let sel_dpad = memory.read(JOYP) & 0b0010_0000 == 0;
        
        self.a = if keys.contains(&Keycode::X) {
            true
        } else { false };
        self.b = if keys.contains(&Keycode::Z) {
            true
        } else { false };
        self.select = if keys.contains(&Keycode::Return) {
            true
        } else { false };
        self.start = if keys.contains(&Keycode::Tab) {
            true
        } else { false };

        self.down = if keys.contains(&Keycode::Down) {
            true
        } else { false };
        self.up = if keys.contains(&Keycode::Up) {
            true
        } else { false };
        self.left = if keys.contains(&Keycode::Left) {
            true
        } else { false };
        self.right = if keys.contains(&Keycode::Right) {
            true
        } else { false };   
        
        let mut input = memory.read(JOYP);
        //println!("{input:#08b}");
        if sel_buttons {
            input = self.buttons_to_bin();
        }
        if sel_dpad {
            input = self.dpad_to_bin();
        }
        memory.write(JOYP, input);

        self.input = input;
    }

    fn buttons_to_bin(&self) -> u8 {
        let mut input = 0b0000_0000;
        if !self.a { input += 0b1 }
        if !self.b { input += 0b10 }
        if !self.select { input += 0b100 }
        if !self.start { input += 0b1000 }
        input
    }

    fn dpad_to_bin(&self) -> u8 {
        let mut input = 0b0000_0000;
        if !self.right { input += 0b1 }
        if !self.left { input += 0b10 }
        if !self.up { input += 0b100 }
        if !self.down { input += 0b1000 }
        input
    }

}
