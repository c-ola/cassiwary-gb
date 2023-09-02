//idk why this is here
pub const XOFFSET: u8 = 3;
pub const YOFFSET: u8 = 0;
pub const REGMASK: u8 = 0b111u8;

//register opcode mappings
pub const B: u8 = 0x0;
pub const C: u8 = 0x1;
pub const D: u8 = 0x2;
pub const E: u8 = 0x3;
pub const H: u8 = 0x4;
pub const L: u8 = 0x5;
pub const A: u8 = 0x7;
pub const READ_HL: u8 = 0x6;

//register view opcode mappings
pub const BC: u8 = 0x0;
pub const DE: u8 = 0x1;
pub const HL: u8 = 0x2;
pub const SP: u8 = 0x3;

//flag bits
pub const FLAG_Z: u8 = 7; // zero flag 
pub const FLAG_N: u8 = 6; // subtraction flag
pub const FLAG_H: u8 = 5; // half-carry flag
pub const FLAG_C: u8 = 4; // full carry flag

// reset memory locations
pub const RST: [u8; 8] = [
    0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38
];


