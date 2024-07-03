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


use core::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Instant, Duration};
use std::cmp::min;
use std::collections::HashSet;
use std::fs::read;
use std::u128;

use interrupts::SERIAL_I;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;


pub const CLOCK_RATE_NANOS: u64 = 238;
pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
pub const LCD_SIZE: usize = LCD_WIDTH*LCD_HEIGHT;
const SCREEN_WIDTH: u32 = LCD_WIDTH as u32 * 4;
const SCREEN_HEIGHT: u32 = LCD_HEIGHT as u32 * 4;

#[derive(Debug, Clone)]
struct PerfError {
    value: String
}

impl fmt::Display for PerfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug)]
struct PerfTimer<const N: usize = 1000> {
    start: Option<Instant>,
    times: [u128; N],
    index: usize,
}

impl<const N: usize> PerfTimer<N> {
    fn new() -> PerfTimer<N> {
        PerfTimer {
            start: None,
            times: [0; N],
            index: 0
        }
    }
    fn start(&mut self) {
        self.start = Some(Instant::now());
    }
    fn end(&mut self) -> Result<(), PerfError> {
        return match self.start {
            None => {
                println!("err");
                Err(PerfError{value: String::from("Must start timer before ending")})
            },
            Some(start) => {
                self.times[self.index] = start.elapsed().as_nanos();
                self.index += 1;
                if self.index >= N {
                    self.index = 0;
                }
                self.start = None;
                Ok(())
            }
        }
    }
    fn avg(&self) -> f64 {
        let sum: u128 = self.times.iter().sum();
        let avg: f64 = sum as f64 / N as f64;
        avg
    }
}



pub struct GBIO {
    timer: HTimer,
    joypad: Joypad,
    pub ppu: PPU,
    //serial: SerialPort,
}

impl GBIO {
    pub fn new() -> GBIO {
        GBIO {
            timer: HTimer::new(),
            joypad: Joypad::default(),
            ppu: PPU::new(),
        }
    }

    pub fn update(&mut self, memory: &mut Memory, keys: &HashSet<Keycode>) -> Result<(), String> {
        self.joypad.update(memory, &keys);
        self.timer.update(false, memory);
        self.ppu.update(memory);
        if memory.read(0xFF02) == 0x81 {
            memory.request_interrupt(SERIAL_I);
            memory.write_io(0xFF02, 0x01);
            print!("{}", memory.read(0xFF01) as char);
            memory.write_io(0xFF01, 0xFF);
        } else if memory.read(0xFF02) == 0x80 {
            //self.gamepack.request_interrupt(SERIAL_I);
            //self.gamepack.write_io(0xFF02, 0x00);
            //self.gamepack.write_io(0xFF01, 0x00);
            //print!("{}", self.gamepack.read(0xFF01) as char);
        }

        Ok(())
    }

}

pub struct GameBoy {
    pub gamepack: Arc<Mutex<Memory>>,
    log_memory: bool,
}

impl GameBoy {

    pub fn new(log_memory: bool) -> GameBoy {
        let memory = Arc::new(Mutex::new(Memory::new(8 * KBYTE)));
        GameBoy {
            gamepack: memory,
            log_memory,
        }
    }

    fn run_cpu(&mut self, stop_signal: Arc<AtomicBool>, clock: Arc<(Mutex<usize>, Condvar)>) -> thread::JoinHandle<()> {
        let mut clock_timer = Instant::now();

        //let mut clock_cycles: usize = 0;
        let mut cpu_cycles: usize = 0;
        let mut counter = 0;

        let instrs = -1;
        let mut cpu = SharpSM83::new();
        // Spawn this
        let memory = Arc::clone(&self.gamepack);
        let cpu_handle = thread::spawn(move || {
            'running: loop {
                println!("cpu: {counter}");
                if counter == instrs || stop_signal.load(Ordering::SeqCst) {
                    break 'running
                }

                let (lock, cvar) = &*clock; 
                let clock_cycles = lock.lock().unwrap();
                let clock_cycles_guard = cvar.wait(clock_cycles).unwrap(); 
                // CPU
                let mut memory = memory.lock().unwrap();
                cpu.update(&mut memory);
                
                
                if cpu_cycles == *clock_cycles_guard {
                    counter += 1;
                    cpu_cycles += cpu.run(&mut memory);
                    println!("cpu: {clock_cycles_guard}");
                }
                clock_timer = Instant::now();
            }
        });
        cpu_handle
    }

    pub fn run_emu(&mut self) -> Result<(), String>{
        /*
         * Setup SDL context and window
         */

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Cassowary Gameboy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA8888, LCD_WIDTH as u32, LCD_HEIGHT as u32)
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)))?;
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut render_timer = Instant::now();

        /*
         * I/O Devices
         */
        let mut io_devices = GBIO::new();
        //let keys: Arc<Mutex<HashSet<Keycode>>> = Arc::new(Mutex::new(HashSet::new()));
        let mut keys: HashSet<Keycode> = HashSet::new();
        let stop = Arc::new(AtomicBool::new(false));

        /*
         * Timing 
         */
        //let clock = Arc::new((Mutex::new(0), Condvar::new()));
        let mut clock_cycles = 0;
        //let cpu_handle = self.run_cpu(stop.clone(), Arc::clone(&clock));
        
        let mut cpu_cycles: usize = 0;
        let mut cpu = SharpSM83::new();

        let mut clock_timer = Instant::now();
        'running: loop {
            if clock_timer.elapsed() > Duration::from_nanos(CLOCK_RATE_NANOS) {
                let mut memory = &mut *self.gamepack.lock().unwrap();
                cpu.update(&mut memory);
                if cpu_cycles == clock_cycles {
                    cpu_cycles += cpu.run(&mut memory);
                }
                // threads enters here
                io_devices.update(memory, &keys).unwrap();
                //let (lock, cvar) = &*clock; 
                //let mut clock_cycles = lock.lock().unwrap();
                clock_cycles += 1;
                //cvar.notify_one();
                //println!("{clock_cycles}");
                clock_timer = Instant::now();
            }
            /*
             * Actual Rendering
             * Event polling is done here to speed up the reset of the code 
             */
            if render_timer.elapsed() > Duration::from_micros(16670){
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} |
                            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                                println!("stopping");
                                //stop.store(true, Ordering::SeqCst);
                                break 'running
                            },
                        _ => {}
                    }
                }

                keys = event_pump
                    .keyboard_state()
                    .pressed_scancodes()
                    .filter_map(Keycode::from_scancode)
                    .collect();
                io_devices.ppu.render(&mut texture)?;

                canvas.copy(&texture, None, None)?;
                canvas.present();

                render_timer = Instant::now();
            }
        }

        //cpu_handle.join().unwrap();
        self.stop();
        Ok(())
    }

    pub fn stop(&self) {    
        if self.log_memory {
            self.log_memory();
            //self.gamepack.lock().unwrap().print(0, 16);
            //self.cpu.print();
        }   
       // println!("Instructions executed: {}", self.cpu.get_instr_executed());
    }

    pub fn load_memory(&mut self, data: &[u8]) {
        for i in 0..data.len() {
            let byte = data[i];
            //self.gamepack.lock().unwrap().write(i as u16, byte);
        }
    }

    pub fn log_memory(&self) {
        match self.gamepack.lock().unwrap().log() {
            Ok(()) => (),
            Err(error) => println!("{error}"),
        }
    }

    /*pub fn peek_cpu(&self) -> &SharpSM83 {
        &self.cpu
    }*/

    pub fn load_rom(&mut self, rom_path: std::path::PathBuf) {

        let logo = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
        ];


        let mut memory = self.gamepack.lock().unwrap();
        match read(rom_path) {
            Ok(buffer) => {
                //rom banks
                for i in 0..buffer.len() {
                    memory.write(i as u16, buffer[i]);
                } 

                // external ram
                if buffer.len() > 0xA000 {
                    for i in 0xA000..0xBFFF {
                        memory.write(i as u16, buffer[i]);
                    }
                }
            }
            Err(error) => match read(BOOT_ROM_PATH) {
                Ok(buffer) => {
                    println!("{error}, trying to recover");
                    //load default boot rom
                    for i in 0..min(buffer.len(), 0x10000) {
                        memory.write(i as u16, buffer[i]);
                    }

                    for i in 0..logo.len() {
                        memory.write(0x0104 + i as u16, logo[i]);
                    }
                },
                Err(error) => panic!("{error} boot rom error, file not found or incorrect file"),
            },
        }




    }

}
