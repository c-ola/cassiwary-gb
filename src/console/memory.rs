pub const KBIT: i32 = 1024;
pub const KBYTE: i32 = 8 * KBIT;

struct ROM {
    size: i32,
}

// size in bits
#[derive(Debug)]
pub enum Memory {
    RAM {size: i32},
    VRAM {size: i32},
    ROM {size: i32}
}

impl Memory {
    //data should be an array i think?
    fn write(&self, loc: i32, size: i32, data: i32){

    }
}

