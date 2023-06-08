pub fn high(x: u16) -> u8 {
    (x >> 8) as u8
}

pub fn low(x: u16) -> u8 {
    x as u8
}

pub fn high_u8(x: u8) -> u8 {
    (x >> 4) as u8
}

pub fn low_u8(x: u8) -> u8 {
    x as u8 - high_u8(x)
}

pub fn u8_to_u16(high: u8, low: u8) -> u16 {
    (high as u16) * (2 << 7) + low as u16
}


