pub mod cpu;
pub mod memory;
pub mod ppu;

use crate::ppu::*;
use crate::cpu::*;
use crate::memory::*;

use std::thread;
use std::sync::mpsc;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::CanvasBuilder;
use std::time::Duration;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect, Point};
use std::collections::HashSet;


use std::time::{Instant};

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

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 960;

pub struct GameBoy {
    instruction_count: u16,
    accumulator: u32,
    cpu: SharpSM83,
    pub gamepack: Memory,
    pub write_idx: u16,
}

impl GameBoy {

    pub fn new() -> GameBoy {
        GameBoy {
            accumulator: 0,
            instruction_count: 10000,
            cpu: SharpSM83::new(),
            gamepack: Memory::new(8 * KBYTE),
            write_idx: 0,
        }
    }

    pub fn init(&mut self) {
    }

    pub fn init_emu(&mut self, rom_path: Option<&str>) {
        //self.cpu.init_emu();
        self.gamepack.init_memory(rom_path);
    }

    pub fn run_emu(&mut self) -> Result<(), String>{
        let mut ppu = PPU::new();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Cassowary Gameboy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .vulkan()
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().unwrap();

        let texture_creator = canvas.texture_creator();

        let mut texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 240, 240)
            .map_err(|e| e.to_string())?;

        canvas.clear();

        canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut prev_keys = HashSet::new();
        let mut start = Instant::now();
        'running: loop {
            
            self.gamepack.write(0xFF00, 0x00);
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            break 'running
                        },
                    _ => {}
                }

            }
            // Create a set of pressed Keys.
            let keys = event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect();

            // Get the difference between the new and old sets.
            let new_keys = &keys - &prev_keys;
            let old_keys = &prev_keys - &keys;

            if !new_keys.is_empty() || !old_keys.is_empty() {
                println!("new_keys: {:?}\told_keys:{:?}", new_keys, old_keys);
            }

           
            if new_keys.contains(&Keycode::X) {
                self.gamepack.write(0xFF00, 0xFF); 
            }


            self.tick_cpu();  


            // only updates the screen 60 times per second
            if start.elapsed() > Duration::new(0, 1_000_000_000u32 / 60) {
                ppu.update(&self.gamepack);
                ppu.render(&mut canvas, &mut texture)?;

                canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
                canvas.present();
                start = Instant::now();
            }            
            // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            prev_keys = keys;
        }

        //self.stop();
        Ok(())
    }


    pub fn set_run_count(&mut self, count: u16) {
        self.instruction_count = count;
    }

    pub fn tick_cpu(&mut self) {

        if !self.cpu.stop {

            match self.cpu.run(&self.gamepack) {
                Ok(()) => {
                    while self.cpu.mem_write_stack.len() > 0 {
                        self.cpu.mem_write -= 1;
                        let dat = self.cpu.mem_write_stack.pop();

                        match dat {
                            Some((x, y)) => self.gamepack.write(y, x),
                            _ => ()
                        }
                    }
                },
                Err(error) => println!("{error}"),
            }

            self.accumulator += 1;
        }

    }

    pub fn stop(&self) {
        self.log_memory();
        self.gamepack.print(0x8000, 100);
    }

    pub fn start(&mut self) {

        println!("Reading instructinos");

        while self.accumulator < self.instruction_count as u32 {
            println!("Instruction: {0}", self.accumulator);
            self.tick_cpu();
        }

    }

    pub fn write(&mut self, data: u8) {
        self.gamepack.write(self.write_idx, data);
        self.write_idx += 1;
    }

    pub fn inc_instr_count(&mut self){
        self.instruction_count += 1;
    }

    pub fn log_memory(&self) {
        match self.gamepack.log() {
            Ok(()) => (),
            Err(error) => println!("{error}"),
        }
    }

    pub fn print_info(&self) {
    }

    pub fn peek_cpu(&self) -> &SharpSM83 {
        &self.cpu
    }

    pub fn peek_memory(&self) -> &Memory {
        &self.gamepack
    }
}
