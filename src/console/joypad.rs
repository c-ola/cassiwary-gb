const JOYP: u16 = 0xFF00;

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

use super::memory::Memory;
use super::regids::IF;

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
    
    buttons: u8,
    dpad: u8,
}

impl Joypad {

    pub fn update(&mut self, memory: &mut Memory, keys: &HashSet<Keycode>) {
        let sel_buttons = memory.read(JOYP) & 0b0010_0000 == 0;
        let sel_dpad = memory.read(JOYP) & 0b0001_0000 == 0;
        
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
        
        let dpad = self.dpad_to_bin();
        let buttons = self.buttons_to_bin();

        if sel_buttons {
            memory.write_io(JOYP, buttons);
            self.buttons = buttons;
        }
        if sel_dpad {
            memory.write_io(JOYP, dpad);
            self.dpad = dpad;
        }

        if self.dpad != dpad {
            println!("dpad: {dpad:#08b}");
            Joypad::request_interrupt(memory);
            self.dpad = dpad;
        }

        if self.buttons != buttons {
            println!("buttons: {buttons:#08b}");
            self.buttons = buttons;
        }
        

    }

    fn buttons_to_bin(&self) -> u8 {
        let mut input = 0b0001_0000;
        if !self.a { input += 0b1 }
        if !self.b { input += 0b10 }
        if !self.select { input += 0b100 }
        if !self.start { input += 0b1000 }
        input
    }

    fn dpad_to_bin(&self) -> u8 {
        let mut input = 0b0010_0000;
        if !self.right { input += 0b1 }
        if !self.left { input += 0b10 }
        if !self.up { input += 0b100 }
        if !self.down { input += 0b1000 }
        input
    }

    fn request_interrupt(memory: &mut Memory) {
        let if_old = memory.read(IF);
        let if_new = if_old | 0b1_0000 ;
        memory.write(IF, if_new);
    }

}
