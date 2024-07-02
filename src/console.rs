pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod timer;
pub mod joypad;
pub mod regids;
pub mod apu;
pub mod interrupts;

use crate::ppu::*;
use crate::cpu::*;
use crate::memory::*;
use crate::timer::*;
use crate::joypad::*;
use crate::regids::*;
use crate::apu::*;


use std::ops::Deref;
use std::time::{Instant, Duration};
use std::cmp::{min, max};
use std::collections::HashSet;
use std::fs::read;

use sdl2::audio::AudioCallback;
use sdl2::audio::AudioSpecDesired;
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
    pub gamepack: Memory,
    pub boot_rom: Memory,
    timer: HTimer,
    joypad: Joypad,
    reset: bool,
    log_memory: bool,
}

impl GameBoy {

    pub fn new(log_memory: bool) -> GameBoy {
        let memory = Memory::new(8 * KBYTE);
        GameBoy {
            gamepack: memory,
            boot_rom: Memory::from_file(0xFF, BOOT_ROM_PATH),
            cpu: SharpSM83::new(),
            timer: HTimer::new(),
            joypad: Joypad::default(),
            reset: false,
            log_memory,

        }
    }

    pub fn tick_cpu(&mut self) -> usize {
        return match self.cpu.run(&mut self.gamepack) {
            Some(cycles) => {
                cycles
            },
            None => panic!("something went wrong"), 
        }
    }

    pub fn run_emu(&mut self) -> Result<(), String>{
        let mut ppu = PPU::new();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Cassowary Gameboy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().unwrap();

        let texture_creator = canvas.texture_creator();

        /*let mut texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGBA8888, 160, 144)
            .map_err(|e| e.to_string())?;*/

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA8888, LCD_WIDTH as u32, LCD_HEIGHT as u32)
            .map_err(|e| e.to_string())?;

        canvas.clear();

        canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
        canvas.present();
        

        // Audio
        
        /*let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: None       // default sample size
        };
        let mut apu = Apu::new();

        let mut device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            apu.get_sound()
        }).unwrap();
        //device.resume();*/

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut render_timer = Instant::now();

        let mut clock_timer = Instant::now();
        let mut log_timer = Instant::now();

        let mut run_time = Instant::now();

        let mut clock_cycles: i32 = 0;
        let mut cpu_cycles: i32 = 0;
        let mut counter = 0;

        let cpu_dur = Duration::from_nanos(238 / 4);

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
            //0x2C7, // call state machine
            0x1,
            0x2,
            0x30C9
        ];

        let mut keys:HashSet<Keycode> = HashSet::new();
        let mut old_keys:HashSet<Keycode> = HashSet::new();

        'running: loop {




            /*
             * Debug Control
             */
            /*if !broken {
              for breakpoint in breakpoints {
              if self.cpu.pc == breakpoint as u16 {
              println!("reached breakpoint, {breakpoint:#04X}");
              broken = true;
              counter += 1;
              break;
              }
              }
              }*/

            if keys.contains(&Keycode::P) {
                //self.cpu.print();
            }
            if keys.contains(&Keycode::S) {
                //self.gamepack.print(self.cpu.get_reg_view(SP) - 0xF, 3)
            }
            if keys.contains(&Keycode::H) {
                //self.gamepack.print(self.cpu.get_reg_view(HL) - 0xF, 3)
            }
            if broken && !old_keys.contains(&Keycode::Return) && keys.contains(&Keycode::Return){
                broken = false;
            }

            if counter == instrs {
                break 'running
            }

            /*
             * Update
             */

            /*
             * Only update input certain times per second to not massively slow down code
             *
             */

            // tick the clock at 4.194 mhz
            if clock_timer.elapsed() > cpu_dur {
                if self.gamepack.read(0xFF02) == 0x81 {
                    let if_old = self.gamepack.read(IF);
                    let if_new = if_old | 0b0_1000 ;
                    self.gamepack.write(IF, if_new);
                    self.gamepack.write_io(0xFF02, 0x01);
                    print!("{}", self.gamepack.read(0xFF01) as char);
                    //self.gamepack.write_io(0xFF01, 0xFF);
                } else if self.gamepack.read(0xFF02) == 0x80 {
                    //let if_old = self.gamepack.read(IF);
                    //let if_new = if_old | 0b0_1000 ;
                    //self.gamepack.write(IF, if_new);
                    //self.gamepack.write_io(0xFF02, 0x00);
                    //self.gamepack.write_io(0xFF01, 0x00);
                }
                //let (if_reg, ie_reg) = (self.read(IF, memory), self.read(IE, memory));
                //println!("if: {:#10b}, ie: {:#10b}", if_reg, ie_reg);

                self.joypad.update(&mut self.gamepack, &keys); 
                self.cpu.update(&mut self.gamepack);
                if cpu_cycles - clock_cycles == 0 {
                    counter += 1;
                    cpu_cycles += self.tick_cpu() as i32;
                }
                //apu.update(&mut self.gamepack);
                if clock_cycles % 456 == 0 {
                    ppu.update(&mut self.gamepack);
                }
                    //apu.update(&mut self.gamepack);

                self.timer.update(self.cpu.stop, &mut self.gamepack);

                clock_cycles += 1;
                clock_timer = Instant::now();
            }


            /*
             * Actual Rendering
             * Event polling is done here to speed up the reset of the code 
             */
            if render_timer.elapsed() > Duration::from_micros(16670){
                //println!("CGB flag: {:#4x}", self.gamepack.read(0x0143));
                // Create a set of pressed Keys.
                //println!("{:#010b}", self.gamepack.read(0xFF00));
               // let mut cb_guard = device.lock();
               // *cb_guard = apu.get_sound();
                //apu.update(&mut self.gamepack);
                //println!("{:#04X}", self.gamepack.read(0xffc5));
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} |
                            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                                break 'running
                            },
                        _ => {}
                    }
                }

                old_keys = keys.clone();
                keys = event_pump
                    .keyboard_state()
                    .pressed_scancodes()
                    .filter_map(Keycode::from_scancode)
                    .collect();
                ppu.render(&mut texture)?;

                canvas.copy(&texture, None, None)?;
                canvas.present();

                render_timer = Instant::now();
            }
            //device.resume();



            if run_time.elapsed() > Duration::from_secs(1){
                // break 'running
            }

        }

        self.stop();
        Ok(())
    }

    pub fn stop(&self) {    
        if self.log_memory {
            self.log_memory();
            self.gamepack.print(0, 16);
            self.cpu.print();
        }   
        println!("Instructions executed: {}", self.cpu.get_instr_executed());
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

    pub fn load_rom(&mut self, rom_path: std::path::PathBuf) {

        let logo = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
        ];


        match read(rom_path) {
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
            Err(error) => match read(BOOT_ROM_PATH) {
                Ok(buffer) => {
                    //load default boot rom
                    for i in 0..min(buffer.len(), 0x10000) {
                        self.gamepack.write(i as u16, buffer[i]);
                    }

                    for i in 0..logo.len() {
                        self.gamepack.write(0x0104 + i as u16, logo[i]);
                    }
                },
                Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
            },
        }




    }

}
