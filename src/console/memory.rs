pub const KBIT: usize = 1024;
pub const KBYTE: usize = 8 * KBIT;

// size in bits
#[derive(Debug)]
pub struct Memory {
    size: usize,
    data: Vec<u8>,
}

impl Memory {

    pub fn new(size: usize) -> Memory {
        Memory {
            size,
            data: vec![0; size],
        }
    }
    
    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    //data should be an array i think?
    pub fn write(&mut self, addr: u16, n: u8){
        self.data[addr as usize] = n;
    }

    pub fn print(&self, x: usize, y: usize){
        for i in x..y {
            println!("{:02X} at addr: {i}", self.data[i]);
        }
    }
}

