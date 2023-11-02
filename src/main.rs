extern crate sdl2;

pub mod console;
pub mod bytes;

use crate::console::*;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Write memory to a file at the end
    #[arg(short='m', long)]
    log_memory: bool,

    /// Path of rom to run
    #[arg(short='p', long, default_value = "")]
    rom_path: std::path::PathBuf,
}

fn main() -> Result<(), String>{
    
    let args = Args::parse();

    let mut gb = GameBoy::new(args.log_memory);
    gb.load_rom(args.rom_path);

    gb.run_emu().unwrap();


    Ok(())
}
