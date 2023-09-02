extern crate sdl2;

pub mod console;
pub mod bytes;

use crate::console::*;

use std::env;

fn parse_arg(args: &[String], opt: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i].starts_with(&opt) {
            let arg = args[i].split_at(opt.len() + 1).1;
            return Some(String::from(arg));
        }
    }
    None
}

fn main() -> Result<(), String>{
    let args: Vec<String> = env::args().collect();
    let rom_path = parse_arg(&args, "path");
    let verbose = parse_arg(&args, "verbose");

    let mut gb = GameBoy::new();
    gb.set_verbose(verbose);
    gb.load_rom(rom_path);

    gb.run_emu().unwrap();


    Ok(())
}
