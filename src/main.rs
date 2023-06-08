pub mod console;
pub mod bytes;

use crate::console::*;

fn main() {
    let mut my_gb = GameBoy::new();
    
    my_gb.print_info();
    my_gb.read_instr();

}

