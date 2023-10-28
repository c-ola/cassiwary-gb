pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod timer;
pub mod regids;

use crate::ppu::*;
use crate::cpu::*;
use crate::memory::*;
use crate::timer::*;
use crate::regids::*;

use std::thread;
use std::sync::mpsc;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::CanvasBuilder;
use std::time::{Instant, Duration};
use std::cmp::min;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect, Point};
use std::collections::HashSet;

use std::fs::read;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
const SCREEN_WIDTH: u32 = LCD_WIDTH as u32 * 4;
const SCREEN_HEIGHT: u32 = LCD_HEIGHT as u32 * 4;

pub struct GameBoy {
    instruction_count: u16,
    accumulator: u32,
    cpu: SharpSM83,
    pub gamepack: Memory,
    pub write_idx: u16,
    rom_path: String,
    verbose: bool,
    has_cartridge: bool,
    timer: HTimer,
    ppu_timer: Instant,
}

impl GameBoy {

    pub fn new() -> GameBoy {
        GameBoy {
            accumulator: 0,
            instruction_count: 10000,
            cpu: SharpSM83::new(),
            gamepack: Memory::new(8 * KBYTE),
            write_idx: 0,
            rom_path: String::new(),
            verbose: false,
            has_cartridge: false,
            timer: HTimer::new(),
            ppu_timer: Instant::now(),

        }
    }

    pub fn init(&mut self, ) {

    }

    pub fn set_verbose(&mut self, v: Option<String>) {
        match v {
            Some(x) => {
                if x == "true" {
                    self.verbose = true;
                }
            },
            None => self.verbose = false,
        }
    }

    pub fn load_rom(&mut self, rom_path: Option<String>) {

        /*for i in 0x104..0x14F {
          self.gamepack.write(i as u16, 0xFF);
          }*/

        match rom_path {
            Some(game_rom) => {
                match read(game_rom) {
                    Ok(buffer) => {
                        //rom banks
                        for i in 0..0x8000 {
                            self.gamepack.write(i as u16, buffer[i]);
                        } 

                        // external ram
                        if buffer.len() > 0xA000 {
                            for i in 0xA000..0xBFFF {
                                self.gamepack.write(i as u16, buffer[i]);
                            }
                        }

                        self.has_cartridge = true;
                    }
                    Err(error) => panic!("{error} no game rom was specified or found"),
                }

            },
            None => match read(BOOT_ROM_PATH) {
                Ok(buffer) => {
                    for i in 0..min(0x8000, buffer.len()) {
                        self.gamepack.write(i as u16, buffer[i]);
                    }
                },
                Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
            },
        };


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
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;

        canvas.clear();

        canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut prev_keys = HashSet::new();
        let mut start = Instant::now();
        let mut ppu_timer = Instant::now();
        let mut render_timer = Instant::now();
        let mut cpu_clock = Instant::now();

        'running: loop {
            start = Instant::now();

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

            self.update(start);

            if ppu_timer.elapsed() > Duration::new(0, (1000000000./59.7) as u32) {
                ppu.request_interrupt(&mut self.gamepack);
                ppu.update(&mut self.gamepack);
                ppu_timer = Instant::now();
            }

            if self.cpu.pc == self.cpu.pc {
                self.tick_cpu();
                //self.cpu.print_info();
            }

            // only updates the screen 60 times per second
            if render_timer.elapsed() > Duration::new(0, (1_000_000_000. / 59.73) as u32){
                ppu.render(&mut canvas, &mut texture)?;

                canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
                canvas.present();
                render_timer = Instant::now();
            }            
            // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            prev_keys = keys;
        }

        self.stop();
        println!("instructions executede: {}", self.accumulator);
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
                //Err(error) => println!("{error}"),
                _ => (), 
            }

            self.accumulator += 1;
        }

    }

    pub fn stop(&self) {    
        if self.verbose {
            self.log_memory();
            self.gamepack.print(0, 100);
        }
    }

    pub fn start(&mut self) {

        println!("Reading instructinos");

        while self.accumulator < self.instruction_count as u32 {
            self.tick_cpu();
        }

    }

    fn update(&mut self, start: Instant){
        //increment div register
        //self.timer.update_div(start);
        let mut div = self.gamepack.read(DIV);
        if start.elapsed() > DIV_DUR {
            match div.overflowing_add(1) {
                (value, false) => div = value,
                (_, true) => div = 0
            }
            self.gamepack.write(DIV, div);
        }

        let mut tima = self.gamepack.read(TIMA);
        let mut tac = self.gamepack.read(TAC);
        let mut tma = self.gamepack.read(TMA);
        let tima_en = (tac & 0b100) != 0;
        let clk_s = tac & 0b11;

        if start.elapsed() > TIMA_DUR[clk_s as usize] && tima_en {
            match tima.overflowing_add(1) {
                (value, false) => tima = value,
                (_, true) => {
                    tima = tma;
                    self.request_interrupt(2);
                }
            }
            self.gamepack.write(TIMA, tima);
            self.gamepack.print(0xFF00, 10);
        }

    }

    fn request_interrupt(&mut self, bit: u8) {
        let if_old = self.gamepack.read(IF);
        let if_new = if_old | (0b1 << bit);
        self.gamepack.write(IF, if_new);
    }

    pub fn load_memory(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            let byte = data[i];
            self.gamepack.write(i as u16, byte);
        }
    }

    pub fn write(&mut self, data: u8) {
        self.gamepack.write(self.write_idx, data);
        self.write_idx += 1;
    }

    pub fn run_n(&mut self, n: u16) {
        self.instruction_count = n;
        self.accumulator = 0;
        self.start();
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
