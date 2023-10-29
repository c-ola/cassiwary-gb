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

use std::time::{Instant, Duration};
use std::cmp::min;
use std::collections::HashSet;
use std::fs::read;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;


pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
pub const LCD_SIZE: usize = LCD_WIDTH*LCD_HEIGHT;
const SCREEN_WIDTH: u32 = LCD_WIDTH as u32 * 4;
const SCREEN_HEIGHT: u32 = LCD_HEIGHT as u32 * 4;

pub struct GameBoy {
    cpu: SharpSM83,
    clock_acc: usize,
    clock: bool,

    pub gamepack: Memory,
    rom_path: String,
    verbose: bool,
    timer: HTimer,
}

impl GameBoy {

    pub fn new() -> GameBoy {
        let memory = Memory::new(8 * KBYTE);
        GameBoy {
            clock_acc: 0,
            clock: false,

            gamepack: memory,
            cpu: SharpSM83::new(),
            rom_path: String::new(),
            verbose: false,
            timer: HTimer::new(),

        }
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

    pub fn tick_cpu(&mut self) {

        if !self.cpu.stop {

            match self.cpu.run(&mut self.gamepack) {
                Some(cycles) => {
                    self.clock_acc = cycles;

                },
                None => panic!("something went wrong"), 
            }
        }

    }

    pub fn run_emu(&mut self) -> Result<(), String>{
        let mut ppu = PPU::new();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Cassowary Gameboy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .opengl()
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
        let mut render_timer = Instant::now();
        let mut cpu_timer = Instant::now();
        let mut counter = 0;
        let instrs = -1;
        
        let mut broken = false;
        let breakpoints = [
            //0x260, //flush wram 1
            //0x272, //
            //0x281, // flush OAM
            //0x28A, // flush RAM 
            //0x293, //copy dam transfer routine into hram
            //0x2A0, // call Flush_BG1
            //0x2A3, // call Sound_Init
            //0x2C4, // main loop
            0x2C7, // call state machine
            0x2C7, // call state machine
            0x8000,
        ];

        'running: loop {

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

            if !broken {
                for breakpoint in breakpoints {
                    if self.cpu.pc == breakpoint as u16 {
                        println!("reached breakpoint, {breakpoint:#04X}");
                        broken = true;
                        counter += 1;
                        break;
                    }
                }
            }

            if new_keys.contains(&Keycode::P) {
                self.cpu.print();
            }

            if broken && new_keys.contains(&Keycode::Return) {
                //broken = false;
            }

            if counter == instrs {
                break 'running
            }

            ppu.update(self.clock_acc, &mut self.gamepack);
            
            if cpu_timer.elapsed() > Duration::new(0, 4194 / 4 as u32) {
                self.clock = true;
                cpu_timer = Instant::now();
            } else {
                self.clock = false;
            }

            if self.clock && (!broken || new_keys.contains(&Keycode::Return)) {
                self.tick_cpu();
            }


            if render_timer.elapsed() > Duration::new(0, (1_000_000_000. / 59.73) as u32){
                ppu.render(&mut canvas, &mut texture)?;

                canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
                canvas.present();

                render_timer = Instant::now();
            }
            prev_keys = keys;
        }

        self.stop();
        Ok(())
    }

    pub fn stop(&self) {    
        if self.verbose {
            self.log_memory();
            self.gamepack.print(0, 16);
            self.cpu.print();
        }   
    }

    /*fn update(&mut self){
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

    }*/

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

    pub fn log_memory(&self) {
        match self.gamepack.log() {
            Ok(()) => (),
            Err(error) => println!("{error}"),
        }
    }

    pub fn peek_cpu(&self) -> &SharpSM83 {
        &self.cpu
    }

    pub fn load_rom(&mut self, rom_path: Option<String>) {

        let logo = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
        ];

        match rom_path {
            Some(game_rom) => {
                match read(game_rom) {
                    Ok(buffer) => {
                        //rom banks
                        for i in 0..buffer.len() {
                            self.gamepack.write(i as u16, buffer[i]);
                        } 

                        // external ram
                        if buffer.len() > 0xA000 {
                            for i in 0xA000..0xBFFF {
                                self.gamepack.write(i as u16, buffer[i]);
                            }
                        }
                    }
                    Err(error) => panic!("{error} no game rom was specified or found"),
                }

            },
            None => eprintln!("No Game Rom Specified"),
        };
        match read(BOOT_ROM_PATH) {
            Ok(buffer) => {
                //load default boot rom
                for i in 0..min(0x8000, buffer.len()) {
                    //self.gamepack.write(i as u16, buffer[i]);
                }

                for i in 0..logo.len() {
                    //self.gamepack.write(0x0104 + i as u16, logo[i]);
                }
            },
            Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
        }

    }

}
