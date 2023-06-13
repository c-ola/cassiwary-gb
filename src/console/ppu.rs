use crate::console::Memory;
use crate::bytes::*;


use sdl2::pixels::Color;
use sdl2::render::{Texture, Canvas};
use sdl2::video::{Window};
use sdl2::rect::Point;


const PALLETTE: [[u8; 4]; 4] = [
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xAA, 0xAA, 0xAA, 0xFF],
    [0x55, 0x55, 0x55, 0xFF],
    [0x00, 0x00, 0x00, 0xFF],
];

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
            self.raw_data[line][0] = memory.read( TB_0 + 16*index + line as u16);
            self.raw_data[line][1] = memory.read( TB_0 + 16*index + line as u16 + 1);
            //let tile_line: u16 = merge_between_u8_u16(self.raw_data[line][1], self.raw_data[line][0]); 
            let a = self.raw_data[line][0];
            let b = self.raw_data[line][1];
            
            for pixel_index in 0..8usize {
                let mask = 0x80 >> pixel_index;
                let _a = (a & mask) >> (7 - pixel_index);
                let _b = (b & mask) >> (7 - pixel_index);
                let id = _b + _a;
               // println!("{a:#010b}, {b:#010b}, {id}");
                let color = PALLETTE[id as usize];
                self.pixel_data[8 * line + pixel_index] = color;
            }
        } 
    }

}


const TM_DIM: (usize, usize) = (32, 32);
const TILEMAP_SIZE: usize = TM_DIM.0 * TM_DIM.1;

const PIXELBUFFER_WIDTH: usize = 240;
const PIXELBUFFER_HEIGHT: usize = 240;
const PIXELBUFFER_SIZE: usize = PIXELBUFFER_WIDTH*PIXELBUFFER_HEIGHT;

const TB_0: u16 = 0x8000;

pub struct PPU {
    bg_map: [Tile; TILEMAP_SIZE],
    fg_map: [Tile; TILEMAP_SIZE],
    bg_px: [[u8; 4]; PIXELBUFFER_SIZE],
    fg_px: [[u8; 4]; PIXELBUFFER_SIZE],

}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            bg_map:[Tile::new(); TILEMAP_SIZE],
            fg_map:[Tile::new(); TILEMAP_SIZE],
            bg_px: [[0; 4]; PIXELBUFFER_SIZE],
            fg_px: [[0; 4]; PIXELBUFFER_SIZE],
        }
    }
    
    pub fn update(&mut self, memory: &Memory){
        for i in 0..TILEMAP_SIZE {
            self.bg_map[i].update(i as u16, memory);
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
