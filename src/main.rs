extern crate sdl2;

pub mod console;
pub mod bytes;

use crate::console::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};

fn main() -> Result<(), String>{
    
    let mut gb = GameBoy::new();
    gb.init();
    gb.run_emu()
}
