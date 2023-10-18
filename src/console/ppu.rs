use crate::console::Memory;
use crate::bytes::*;

use sdl2::pixels::Color;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;
use sdl2::rect::Point;

use super::regids::IF;

const PALLETTE: [[u8; 4]; 4] = [    
    [0xff, 0xff, 0xff, 0xFF],
    [0xaa, 0xaa, 0xaa, 0xFF],
    [0x55, 0x55, 0x55, 0xFF],
    [0x00, 0x00, 0x00, 0xFF],

];

const VB_0: u16 = 0x8000; // used when lcdc bit 7 = 1
const VB_1: u16 = 0x8800;
const VB_2: u16 = 0x9000; // objects only

//tile map areas
const TMA_0: u16 = 0x9800;
const TMA_1: u16 = 0x9C00;

//registsers 
const LCDC: u16 = 0xFF40;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF46;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
//const BGPS: u16 = 0xFF68;

// bit maks for each flag
const LCD_EN: u8 = 1 << 7;
const WIN_TM: u8 = 1 << 6;
const WIN_EN: u8 = 1 << 5;
const BGWIN_TILES: u8 = 1 << 4;
const BG_TM: u8 = 1 << 3;
const OBJ_S: u8 = 1 << 2;
const OBJ_EN: u8 = 1 << 1;
const BGWIN_EN: u8 = 1 << 0;

// bit masks for STAT reg
const LYC_INT_SEL: u8 = 1 << 6;
const M2_INT_SEL: u8 = 1 << 5;
const M1_INT_SEL: u8 = 1 << 4;
const M0_INT_SEL: u8 = 1 << 3;
const LYC_EQ_LY: u8 = 1 << 2;
const PPU_MODE: u8 = 0b11;

// masks for BGP (palette)
const ID3: u8 = 0xC0;
const ID2: u8 = 0x30;
const ID1: u8 = 0x0C;
const ID0: u8 = 0x03;
const PALETTE: [u32; 4] = [0xFFFFFFFF, 0xBBBBBBFF, 0x666666FF, 0x000000FF];

#[derive(Copy, Clone)]
struct Tile {
    raw_data: [[u8; 2]; 8], 
    pub pixel_data: [[u8; 4]; 8*8],
}

impl Tile {
    pub fn new() -> Tile{
        Tile {
            raw_data: [[0x33; 2]; 8],
            pixel_data: [[0; 4]; 8*8],
        }
    }
    
    pub fn update(&mut self, index: u16, memory: &Memory) {
        for line in 0..8 {
            self.raw_data[line][0] = memory.read(index + line as u16);
            self.raw_data[line][1] = memory.read(index + line as u16 + 1);

            let a = self.raw_data[line][0];
            let b = self.raw_data[line][1];
            
            for pixel_index in 0..8usize {
                let mask = 0x80 >> pixel_index;
                let _a = (a & mask) >> (7 - pixel_index);
                let _b = (b & mask) >> (7 - pixel_index);
                let id = (_b << 1) + _a;
                let color = PALLETTE[id as usize];
                self.pixel_data[8 * line + pixel_index] = color;
            }
        } 
    }

}

//https://gbdev.io/pandocs/pixel_fifo.html

struct Pixel {
    color: u8,
    palette: u8,
    sprite_prio: u8,
    bg_prio: u8,
}

const TM_DIM: (usize, usize) = (32, 32);
const TILEMAP_SIZE: usize = TM_DIM.0 * TM_DIM.1;

const PIXELBUFFER_WIDTH: usize = 240;
const PIXELBUFFER_HEIGHT: usize = 240;
const PIXELBUFFER_SIZE: usize = PIXELBUFFER_WIDTH * PIXELBUFFER_HEIGHT;


use std::collections::VecDeque;

pub struct PPU {
    ly: u8,
    scx: u8,
    bg_fifo: VecDeque<Pixel>,
    lcdc: u8,
    bg_map: [Tile; TILEMAP_SIZE],
    w_map: [Tile; TILEMAP_SIZE],
    bg_px: [[u8; 4]; PIXELBUFFER_SIZE],
    w_px: [[u8; 4]; PIXELBUFFER_SIZE],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            ly: 10,
            scx: 0,
            bg_fifo: VecDeque::new(),
            lcdc: 0,
            bg_map:[Tile::new(); TILEMAP_SIZE],
            w_map:[Tile::new(); TILEMAP_SIZE],
            bg_px: [[0; 4]; PIXELBUFFER_SIZE],
            w_px: [[0; 4]; PIXELBUFFER_SIZE],
        }
    }
    
    fn updateCopyRegisters(&mut self, memory: &Memory) {
        
    }

    pub fn request_interrupt(&mut self, memory: &mut Memory) {
        let if_old = memory.read(IF);
        let if_new = if_old | 0b1 ;
        memory.write(IF, if_new);
    }
    
    fn getTile(&mut self, memory: &mut Memory){
        let mut tilemap = TMA_0;
        

    }

    fn getTileHigh(&mut self, memory: &mut Memory){
    }

    fn getTileLow(&mut self, memory: &mut Memory){
    }
    

    pub fn update(&mut self, memory: &mut Memory){
        
        self.lcdc = memory.read(LCDC);

        if self.ly < 153 {
            self.ly += 1;
        }else {
            self.ly = 0;
        }
        memory.write(0xFF44, 0x10);


        let enable = bit!(self.lcdc, 7) != 0;

        let w_vram_bank = match bit!(self.lcdc, 4) {
            0u8 => VB_0,
            _ => VB_1
        };

        let bg_vram_bank = match bit!(self.lcdc, 4) {
            0u8 => VB_1,
            _ => VB_2
        };

        let w_tma = match bit!(self.lcdc, 6) {
            0 => TMA_0,
            _ => TMA_1
        };

        let bg_tma = match bit!(self.lcdc, 4) {
            0 => TMA_0,
            _ => TMA_1
        };

        for i in 0..TILEMAP_SIZE{
            let index = memory.read(0x9800 + i as u16) as u16;
            self.bg_map[i].update(index + VB_0, memory);
            //self.bg_map[i].update(i as u16 * 16 + 0x8000, memory);
            //self.w_map[i].update(index + vram_bank, memory);
            //self.w_map[i].update(index + , memory);
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, texture: &mut Texture) -> Result<(), String> {

        let _result = canvas.with_texture_canvas(texture, |texture_canvas|{

            texture_canvas.set_draw_color(Color::BLACK);
            texture_canvas.clear();

            for tile_idx in 0..TILEMAP_SIZE {
                let t_x = tile_idx % 32;
                let t_y = tile_idx / 32;

                let pixel_data = self.bg_map[tile_idx].pixel_data;

                for i in 0..64 {
                    let color = pixel_data[i];
                    let color = Color::RGBA(color[0], color[1], color[2], color[3]);
                    let x = 8 * t_x + i % 8;
                    let y = 8 * t_y + i / 8;
                    texture_canvas.set_draw_color(color);
                    texture_canvas.draw_point(Point::new(x as i32, y as i32)).expect("cant draw point");
                }

            }

        });

        Ok(())
    }


}
