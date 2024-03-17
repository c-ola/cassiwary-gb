pub fn set_bit(x: u8, bit: u8, value: bool) -> u8 {
    match value {
        true => {x | (0b1 << bit)},
        false => {x & !(0b1 << bit)}
    }
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
    x >> 4
}

pub fn low_u8(x: u8) -> u8 {
    x & 0x0F
}

pub fn has_bit_u8(n: u8, i: u8) -> bool {
    n & (0b1 >> i) != 0
}

pub fn has_bit_u16(n: u16, i: u16) -> bool {
    n & (0b1 >> i) != 0
}

pub fn make_flag(result: u8, n_flag: bool, half_c: bool, full_c: bool) -> u8 {
    let mut new_flag = 0u8;
    new_flag += if result == 0 { 0b1000_0000 } else { 0 };
    new_flag += if n_flag { 0b0100_0000 } else { 0 };
    new_flag += if half_c { 0b0010_0000 }  else { 0 };
    new_flag += if full_c { 0b0001_0000 } else { 0 };
    new_flag 
}

pub fn i16_add(a: i16, b: i16) -> (i16, bool, bool) {
    let result = a.overflowing_add(b);
    (
        result.0, 
        (a & 0xFF) + (b & 0xFF) > 0xFF,
        (a & 0xF) + (b & 0xF) > 0xF,
    )
}

// fix the carry bits here
// .0 is the result
// .1 is the carry flag
// .2 is the half carry flag
pub fn u16_add(a: u16, b: u16) -> (u16, bool, bool) {
    let result = a.overflowing_add(b);
    (
        result.0,
        result.1,
        (a & 0xFFF) + (b & 0xFFF) > 0xFFF,
    )
}

pub fn u16_sub(a: u16, b: u16) -> (u16, bool, bool) {
    let result = a.overflowing_sub(b);
    (
        result.0,
        result.1,
        (a & 0xF).overflowing_sub(b & 0xF).1
    )
}

//return: (sum, flag), 
pub fn u8_add(a: u8, b: u8) -> (u8, u8) {
    let result = a.overflowing_add(b);
    let full_c = result.1;
    let half_c = (a & 0xF) + (b & 0xF) > 0xF;
    let flag = make_flag(result.0, false, half_c, full_c);
    (result.0, flag)
}

pub fn u8_addc(a: u8, b: u8, c: u8) -> (u8, u8) {
    let s1 = b.overflowing_add(c);

    let result = a.overflowing_add(s1.0);
    let half_c = (a & 0xF) + (b & 0xF) + c > 0xF;
    let full_c = result.1 || s1.1;
    let flag = make_flag(result.0, false, half_c, full_c);
    (result.0, flag)
}

pub fn u8_sub(a: u8, b: u8) -> (u8, u8) {
    let result = a.overflowing_sub(b);
    let half_c = (b & 0xF) > (a & 0xF);
    let full_c = b > a;
    let flag = make_flag(result.0, true, half_c, full_c);
    (result.0, flag)
}

pub fn u8_subc(a: u8, b: u8, c: u8) -> (u8, u8) {
    let s1 = b.overflowing_add(c);
    let result = a.overflowing_sub(s1.0);
    let half_c = (b & 0xF) + c > (a & 0xF);
    let full_c = result.1 || s1.1;
    let flag = make_flag(result.0, true, half_c, full_c);
    (result.0, flag)
}

pub fn u8_and (a: u8, b: u8) -> (u8, u8) {
    let result = a & b;
    let flag = make_flag(result, false, true, false);
    (result, flag)
}

pub fn u8_xor (a: u8, b: u8) -> (u8, u8) {
    let result = a ^ b;
    let flag = make_flag(result, false, false, false);
    (result, flag)
}

pub fn u8_or (a: u8, b: u8) -> (u8, u8) {
    let result = a | b;
    let flag = make_flag(result, false, false, false);
    (result, flag)
}

pub fn u8_cmp (a: u8, b: u8) -> (u8, u8) {
    u8_sub(a, b)
}

/// msb, lsb
pub fn u8_to_u16(msb: u8, lsb: u8) -> u16 {
    lsb as u16 + msb as u16 * (2 << 7) 
}

