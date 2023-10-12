use std::time::Duration;

/*  
 *  CPU register ids
 *
 *
 */
pub const B: u8 = 0x0;
pub const C: u8 = 0x1;
pub const D: u8 = 0x2;
pub const E: u8 = 0x3;
pub const H: u8 = 0x4;
pub const L: u8 = 0x5;
pub const A: u8 = 0x7;
pub const READ_HL: u8 = 0x6;

pub const BC: u8 = 0x0;
pub const DE: u8 = 0x1;
pub const HL: u8 = 0x2;
pub const SP: u8 = 0x3;

/*
 *
 */


/*  Interrupt Register addresses and
 *  important constants for interrupt handling
 *
 */

pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

pub const DIV_RATE: u32 = 16384;
pub const DIV_DUR: Duration = Duration::new(0, 1_000_000_000u32 / DIV_RATE);
pub const DIV: u16 = 0xFF04;

pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;
pub const TIMA_DUR: [Duration; 4] = [
    Duration::new(0, 1_000_000_000u32 / 4096),
    Duration::new(0, 1_000_000_000u32 / 262144),
    Duration::new(0, 1_000_000_000u32 / 65536),
    Duration::new(0, 1_000_000_000u32 / 16384),
];

pub const VBLANK: u8 = 40;
pub const VBLANK_RATE: f32 = 59.7;


/*
 *  Memory Registers addresses
 */

