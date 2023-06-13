extern crate sdl2;

pub mod console;
pub mod bytes;

use crate::console::*;

use std::env;


fn parse_config(args: &[String]) -> Option<&str> {
    println!("{}", args.len());
    return if args.len() < 2 {
        None
    }else {
        Some(&args[1])
    }
}

fn main() -> Result<(), String>{
    let args: Vec<String> = env::args().collect();
    let rom_path = parse_config(&args);

    let mut gb = GameBoy::new();
    gb.init(rom_path);
    gb.run_emu()
}
