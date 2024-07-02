const JOYP: u16 = 0xFF00;

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

use crate::interrupts::JOYPAD_I;

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
        
        self.a = if keys.contains(&Keycode::X) {
            true
        } else { false };
        self.b = if keys.contains(&Keycode::Z) {
            true
        } else { false };
        self.select = if keys.contains(&Keycode::Backspace) {
            true
        } else { false };
        self.start = if keys.contains(&Keycode::Return) {
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
        

        let joyp = memory.read_io(JOYP);
        let sel_buttons = joyp & 0b0010_0000 == 0;
        let sel_dpad = joyp & 0b0001_0000 == 0;
        
        if sel_buttons | sel_dpad {
            //println!("joyp: {:#010b}", joyp);
        }
        let dpad = self.dpad_to_bin();
        let buttons = self.buttons_to_bin();
        
        let mut data = joyp & 0xF0;
        if sel_buttons {
            data |= buttons;
        }
        else if sel_dpad {
            data |= dpad;
        } else {
            data |= 0x0F;
        }
        //memory.request_interrupt(JOYPAD);

        if self.dpad != dpad {
            println!("dpad: {data:#010b}");
            memory.request_interrupt(JOYPAD_I);
            self.dpad = dpad;
        }
        if self.buttons != buttons {
            println!("buttons: {data:#010b}");
            self.buttons = buttons;
        }
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
