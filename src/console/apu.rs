use core::f32;

use crate::{bytes::u8_to_u16, console::*};
use super::regids::*;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

// 7 = audio on/off, 3 = ch4 on, 2 = ch3 on 1 = ch2 on, 0 = ch11 on
pub const NR52: u16 = 0xFF26;
pub const NR51: u16 = 0xFF25;
pub const NR50: u16 = 0xFF24;

// channel 1
pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;


// channel 3
pub const NR30: u16 = 0xFF1B;
pub const NR31: u16 = 0xFF1C;
pub const NR32: u16 = 0xFF1D;
pub const NR33: u16 = 0xFF1E;
pub const NR34: u16 = 0xFF1F;

pub const DUTY_CYCLES: [f32; 4] = [ 0.125, 0.25, 0.50, 0.75 ];

// 4 audio channels NRxy;
#[derive(Debug, Default)]
pub struct Apu {
    period1: u16,
    volume1: u8,
    phase: u16,
    channel1: Vec<u8>,
    channel3: Vec<u8>,
    channel3_pos: u8,
    sweeping: bool,
    /*nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8*/
}

impl Apu {
    pub fn new() -> Apu {
        Apu::default()
    }

    pub fn update(&mut self, memory: &mut Memory) {
        self.sweeping = true;
        //channel 1
        let nr10 = memory.read(NR10);
        let pace = nr10 & 0b01110000 >> 4;
        let direction = nr10 & 0b00001000 >> 3;
        let individual_step = nr10 & 0b00000111;
        
        let nr11 = memory.read(NR11);
        let wave_duty = nr11 & 0b11000000 >> 6;
        let initial_length = nr11 & 0b00011111;

        let nr12 = memory.read(NR12);
        let initial_volume = nr12 & 0b11110000 >> 4;
        let env_dir = nr12 & 0b00001000 >> 3;
        let sweep_pace = nr12 & 0b00000111;

        let period_low = memory.read(NR13);

        let nr14 = memory.read(NR14);
        let trigger = nr14 & 0b10000000 >> 7;
        let length_en = nr14 & 0b01000000 >> 6;
        let period_high = nr14 & 0b00000111;

        self.volume1 = initial_volume;
        self.period1 = u8_to_u16(period_high, period_low);

        // if pace > hz
        self.period1 = if direction == 0 {
            self.period1 + self.period1 / 2u16.pow(individual_step as u32)
        } else {
            self.period1 - self.period1 / 2u16.pow(individual_step as u32)
        };

        self.phase = (DUTY_CYCLES[wave_duty as usize] * initial_length as f32) as u16;
        if self.sweeping {
            let x = if self.period1 <= self.phase {
                self.volume1
            } else {
                0
            };

            self.channel1.push(x);
        }

        for i in 0..16 {
            let x = memory.read(0xFF30 + i);
            self.channel3.push(x & 0xF0 >> 4);
            self.channel3.push(x & 0x0F);
        }
    }

    pub fn get_sound(&mut self) -> Sound {
        let mut data: Vec<u8> = Vec::new();
        let data1 = self.channel1.clone();
        let data3 = self.channel3.clone();
        for i in 0..min(data1.len(), data3.len()){
            data.push(data1[i] + data3[i]);
        }
        self.channel1.clear();
        self.channel3.clear();
        Sound {
            data: data3,
            pos: 0
        }
    } 

}

pub struct Sound {
    data: Vec<u8>,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for dst in out.iter_mut() {
            *dst = *self.data.get(self.pos).unwrap_or(&128);
            self.pos += 1;
        }
    }
}


pub struct SquareWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
