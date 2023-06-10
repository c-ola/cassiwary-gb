use bitvec::prelude::*;

#[macro_export]
macro_rules! bitsliceu8 {
    ( $elem:expr ) => {
        {
            let bits = BitSlice::<u8, Msb0>::from_element($elem);
            bits
        }
    };
}

pub fn cmpbit(x: u8, y: u8) -> bool {
    x & y == y
}

pub fn maskbits(x: u8, y: u8) -> u8 {
    x & y
}

pub fn high_u16(x: u16) -> u8 {
    (x >> 8) as u8
}

pub fn low_u16(x: u16) -> u8 {
    x as u8
}

pub fn high_u8(x: u8) -> u8 {
    (x >> 4) as u8
}

pub fn low_u8(x: u8) -> u8 {
    x - (high_u8(x) << 4)
}

pub fn has_bit_u8(n: u8, i: u8) -> bool {
    n & (0b1 >> i) != 0
}

pub fn has_bit_u16(n: u16, i: u16) -> bool {
    n & (0b1 >> i) != 0
}

pub fn make_flag(result: u8, carry_bits: u8, n: u8) -> u8 {
    let mut new_flag = 0u8;
    new_flag += if result == 0 { 0b1000_0000 } else { 0 };
    new_flag += n;
    new_flag += if carry_bits & 0b0000_1000 != 0 { 0b0010_0000 } else { 0 }; // 6 is hflag bit spot
    new_flag += if carry_bits & 0b1000_0000 != 0 { 0b0001_0000 }  else { 0 };
    new_flag
}

pub fn make_flag_2(result: u8, full_c: bool, half_c: bool, n_flag: u8) -> u8 {
    let mut new_flag = 0u8;
    new_flag += if result == 0 { 0b1000_0000 } else { 0 };
    new_flag += n_flag;
    new_flag += if full_c { 0b0001_0000 } else { 0 };
    new_flag += if half_c { 0b0010_0000 }  else { 0 };
    new_flag 
}

pub fn i16_add(a: i16, b: i16) -> (u16, bool, bool) {
    (
        (a + b) as u16, 
        has_bit_u16(a as u16, 15) && has_bit_u16(b as u16, 15),
        has_bit_u16(a as u16, 11) && has_bit_u16(b as u16, 11)
    )
}

// fix the carry bits here
pub fn u16_add(a: u16, b: u16) -> (u16, bool, bool) {
    (
        (a as u32 + b as u32) as u16, 
        has_bit_u16(a, 15) && has_bit_u16(b, 15),
        has_bit_u16(a, 11) && has_bit_u16(b, 11)
    )
}

pub fn u16_sub(a: u16, b: u16) -> (u16, bool, bool) {
    (
        (a as u32 + !b as u32) as u16,
        has_bit_u16(a, 15) && !has_bit_u16(b, 15),
        has_bit_u16(a, 11) && !has_bit_u16(b, 11)
    )
}

pub fn u8_add(a: u8, b: u8) -> (u8, u8) {
    let mut result = ((a as u16 + b as u16) as u8, 0u8);
    let half_c = (a & 0xF) > 0xF -(b & 0xF);
    let full_c = a > 0xFF - b;

    result.1 = make_flag_2(result.0, full_c, half_c, 0);

    result
}

pub fn u8_sub(a: u8, b: u8) -> (u8, u8) {
    let mut result = (((a as u16 + !b as u16) as u8) >> 1, 0u8);
    let half_c = low_u8(b) > low_u8(a);
    let full_c = b > a;

    result.1 = make_flag_2(result.0, full_c, half_c, 0b0100_0000);

    result
}

pub fn u8_and (a: u8, b: u8) -> (u8, u8) {
    let mut result = (a & b, 0u8);

    result.1 = 0b0010_0000 + if result.0 == 0 {0b1000_000} else { 0b0};

    result
}

pub fn u8_xor (a: u8, b: u8) -> (u8, u8) {
    let mut result = (a ^ b, 0u8);

    result.1 = if result.0 == 0 {0b1000_000} else { 0b0};

    result
}

pub fn u8_or (a: u8, b: u8) -> (u8, u8) {
    let mut result = (a | b, 0u8);

    result.1 = 0b0000_0000 + if result.0 == 0 {0b1000_000} else { 0b0};

    result
}

pub fn u8_cmp (a: u8, b: u8) -> (u8, u8) {
    u8_sub(a, b)
}


pub fn u8_to_u16(lsb: u8, msb: u8) -> u16 {
    (lsb as u16) * (2 << 7) + msb as u16
}

