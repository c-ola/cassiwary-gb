const JOYP: u16 = 0xFF00;

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

use crate::{interrupts::JOYPAD_I, test_bit};

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
    buttons: u8,
    dpad: u8,
}

impl Joypad {

    pub fn update(&mut self, memory: &mut Memory, keys: &HashSet<Keycode>) {
        
        self.a = keys.contains(&Keycode::X);
        self.b = keys.contains(&Keycode::Z);
        self.select = keys.contains(&Keycode::Backspace);
        self.start = keys.contains(&Keycode::Return);
        self.down = keys.contains(&Keycode::Down);
        self.up = keys.contains(&Keycode::Up);
        self.left = keys.contains(&Keycode::Left);
        self.right = keys.contains(&Keycode::Right);

        let joyp = memory.read(JOYP);
        let sel_buttons = !test_bit!(joyp, 5);
        let sel_dpad = !test_bit!(joyp, 4);

        let dpad = self.dpad_to_bin();
        let buttons = self.buttons_to_bin();
        
        let mut data = joyp & 0xF0;
        if sel_buttons {
            data |= buttons;
        }
        if sel_dpad {
            data |= dpad;
        }
        if !sel_dpad && !sel_buttons {
            data |= 0x0F;
        }
        //memory.request_interrupt(JOYPAD);

        if self.dpad != dpad {
            println!("dpad: {dpad:#010b}");
            memory.request_interrupt(JOYPAD_I);
            self.dpad = dpad;
        }
        if self.buttons != buttons {
            println!("buttons: {buttons:#010b}");
            memory.request_interrupt(JOYPAD_I);
            self.buttons = buttons;
        }
        //memory.request_interrupt(JOYPAD_I);
        memory.write_io(JOYP, data);
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
