pub mod console;
pub mod bytes;

use crate::console::*;

fn main() {
    let mut my_gb = GameBoy::new();
    my_gb.init();
    my_gb.set_run_count(44);
    my_gb.start();
    my_gb.gamepack.print(0, 10);
}
