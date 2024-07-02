use super::regids::IF;
use crate::console::Memory;
use crate::{console::*, test_bit};

use interrupts::{STAT_I, VBLANK_I};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

use std::collections::VecDeque;

const VB_0: u16 = 0x8000;
const VB_1: u16 = 0x8800;
const VB_2: u16 = 0x9000;

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
const DMA: u16 = 0xFF46;
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

/*const PALETTE: [Color; 4] = [
Color::RGB(0xe0, 0xf8, 0xd0),
Color::RGB(0x88, 0xc0, 0x70),
Color::RGB(0x34, 0x68, 0x56),
Color::RGB(0x08, 0x18, 0x20),
];*/

const PALETTE: [[u8; 4]; 4] = [
    [0xE0, 0xF8, 0xD0, 0xFF],
    [0x88, 0xC0, 0x70, 0xFF],
    [0x34, 0x68, 0x56, 0xFF],
    [0x08, 0x18, 0x20, 0xFF],
];

const PALETTE_OBJ: [[u8; 4]; 4] = [
    [0x00, 0x00, 0x00, 0x00],
    [0x88, 0xC0, 0x70, 0xFF],
    [0x34, 0x68, 0x56, 0xFF],
    [0x08, 0x18, 0x20, 0xFF],
];

//https://gbdev.io/pandocs/pixel_fifo.html
#[derive(Debug)]
struct Pixel {
    pub color: u8,
    pub palette: u8,
    pub sprite_prio: u8,
    pub bg_prio: u8,
}

impl Pixel {
    pub fn new(color: u8, palette: u8, sprite_prio: u8, bg_prio: u8) -> Pixel {
        Pixel {
            color,
            palette,
            sprite_prio,
            bg_prio,
        }
    }
}

pub struct PPU {
    dots: usize,
    mode: u8,

    //fetcher
    fx: u8,
    fy: u8,

    // registers
    lcdc: u8,
    ly: u8,
    lyc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,

    bg_fifo: VecDeque<Pixel>,
    obj_fifo: VecDeque<Pixel>,
    bg: [u8; LCD_SIZE * 4],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            dots: 0,
            mode: 0,

            fx: 0,
            fy: 0,

            lcdc: 0u8,
            ly: 0u8,
            lyc: 0u8,
            stat: 0u8,
            scy: 0u8,
            scx: 0u8,
            wy: 0u8,
            wx: 0u8,
            bgp: 0u8,
            obp0: 0u8,
            obp1: 0u8,

            bg_fifo: VecDeque::new(),
            obj_fifo: VecDeque::new(),
            bg: [0x00; LCD_SIZE * 4],
        }
    }

    fn update_registers(&mut self, memory: &Memory) {
        self.lcdc = memory.read(LCDC);
        self.lyc = memory.read(LYC);
        self.stat = memory.read(STAT);
        self.scy = memory.read(SCY);
        self.scx = memory.read(SCX);
        self.wy = memory.read(WY);
        self.wx = memory.read(WX);
        self.bgp = memory.read(BGP);
        self.obp0 = memory.read(OBP0);
        self.obp1 = memory.read(OBP1);
    }

    fn set_registers(&mut self, memory: &mut Memory) {
        memory.write(LY, self.ly);
    }

    fn get_tile(&mut self, memory: &mut Memory) -> u16 {
        //in_window is not working :)
        let in_window = self.fx < self.wx / 8
            && self.fx + 1 >= self.wx / 8
            && self.fy >= self.wy
            && self.fy <= 143;

        let win_tma = if self.check_lcdc(WIN_TM) {
            TMA_0
        } else {
            TMA_1
        };

        let bg_tma = if !self.check_lcdc(BG_TM) {
            TMA_0
        } else {
            TMA_1
        };

        let tma = if self.check_lcdc(WIN_EN) && in_window {
            win_tma
        } else {
            bg_tma
        };
        self.scy = 0;
        self.scx = 0;
        self.fx = (self.fx + self.scx / 8) & 0x1F;
        //self.fy = self.scy;
        self.fy = ((self.ly as u16 + self.scy as u16) & 0xFF) as u8;

        if in_window {
            //self.fx = (self.fx + self.wx.overflowing_sub(7).0 / 8) & 0x1F;
            //self.fy = ((self.ly as u16 + self.scy as u16 + self.wy as u16) & 0xFF) as u8;
        }

        let block_y = self.fy as u16 / 8;
        let loc = tma + self.fx as u16 + 32 * block_y;
        //println!("{loc:#04X}, fetcher: {}, {}", self.fx, block_y);

        memory.read(loc) as u16
    }

    fn get_obj(&mut self, memory: &mut Memory) -> Vec<u16> {
        let mut valid_objects = Vec::new();
        let ly = memory.read(LY) as u16;
        // 0 for 1 tile, 1 for 2 tiles
        let range = if !self.check_lcdc(OBJ_S) { 8 } else { 16 };
        for addr in (0xFE00..0xFE9F).step_by(4) {
            let y = memory.read(addr) as u16;
            let y_max = y + range as u16;
            //means the object is on the current scanline
            if ly + 16 >= y && ly + 16 < y_max {
                valid_objects.push(addr);
            }
        }
        valid_objects
    }

    fn mix_bytes(low: u8, high: u8) -> [u8; 8] {
        let mut pixels = [0u8; 8];

        for i in 0..8 {
            let bit = 0x1 << i as u8;
            let lsb = (low & bit) >> i;
            let msb = (high & bit) >> i;
            pixels[7 - i] = lsb + (msb << 1);
        }

        pixels
    }

    fn mix_bytes_obj(low: u8, high: u8, y_flip: bool, x_flip: bool) -> [u8; 8] {
        let mut pixels = [0u8; 8];

        for i in 0..8 {
            let bit = 0x1 << i as u8;
            let lsb = (low & bit) >> i;
            let msb = (high & bit) >> i;
            pixels[7 - i] = lsb + (msb << 1);
        }

        if x_flip {
            pixels.reverse()
        }
        pixels
    }

    pub fn update(&mut self, memory: &mut Memory) {
        self.update_registers(memory);

        if !self.check_lcdc(BGWIN_EN) {
            //println!("here")
        }

        if !self.check_lcdc(LCD_EN) {
            //self.clear();
            //return
        }

        if self.ly == self.ly && self.stat & 0b01000000 != 0 {
            memory.request_interrupt(STAT_I);
            self.stat |= 0b10;
        }
        //if self.dots % 20 == 0 {
        self.fx = 0;
        //}
        for _ in 0..20 {
            self.dot(memory);
        }
        //if self.dots % 20 == 0 {
        self.ly = (self.ly + 1) % 154;
        //}
        self.dots += 1;

        self.set_registers(memory);
    }

    /*
     *
     * Modify scan line to go pixel by pixel, this means that mixing bytes will have to be changed
     * A way to index 2 bits from bytes will be needed
     * for x in scanlineLength:
     *  fx = x + scx / 8
     *  fy = y + scy
     *
     *  etc
     */
    fn dot(&mut self, memory: &mut Memory) {
        if self.ly <= 143 {
            // get valid objects to be drawn
            self.mode = 2;
            let mut objects = self.get_obj(memory);
            objects.sort_by(|a, b| {
                let xa = memory.read(a + 1).overflowing_sub(8).0;
                let xb = memory.read(b + 1).overflowing_sub(8).0;
                xa.partial_cmp(&xb).unwrap()
            });

            if self.stat & 0b0010_0000 != 0 {
                memory.request_interrupt(STAT_I);
            }

            self.mode = 3;

            let mut obj_counter = 0;
            //render each tile
            //for i in 0..20 {
            let tile_index = self.get_tile(memory);
            let mut vram_bank = if test_bit!(self.lcdc, 4) {
                if tile_index < 128 {
                    VB_0
                } else {
                    VB_0 // should be VB_1 i think
                }
            } else {
                if tile_index < 128 {
                    VB_2
                } else {
                    VB_0
                }
            };

            if self.lcdc & WIN_EN != 0 {
                vram_bank = if self.lcdc & BGWIN_TILES != 0 {
                    VB_0
                } else {
                    VB_1
                };
            }
            let index_low = vram_bank + tile_index * 16 + (self.fy as u16 % 8) * 2;
            let index_high = index_low + 1;
            let low = memory.read(index_low);
            let high = memory.read(index_high);
            let bgp = memory.read(BGP);
            let pixels = PPU::mix_bytes(low, high);
            for p in 0..8 {
                self.bg_fifo.push_back(Pixel::new(pixels[p], bgp, 0, 0));
            }

            if obj_counter < 10 {
                for addr in &objects {
                    //let y = memory.read(addr + 0);
                    let x = memory.read(*addr + 1);
                    let x_min = self.fx as u16 * 8 + 8;
                    let x_max = x_min + 8;
                    if x as u16 >= x_min && (x as u16) < x_max {
                        //obj_counter += 1;
                        let attributes = memory.read(*addr + 3);
                        let y_flip = test_bit!(attributes, 6);
                        let x_flip = test_bit!(attributes, 5);

                        let obj_ti = memory.read(*addr + 2) as u16;

                        let y = memory.read(*addr) as u16;
                        let diff =
                            ((self.fy as u16).overflowing_sub(y).0.overflowing_add(16).0) % 8;
                        let obj_internal_y = if !y_flip { diff * 2 } else { (8 - diff) * 2 };

                        let index_low = VB_0 + obj_ti * 16 + obj_internal_y;
                        let index_high = index_low + 1;
                        let low = memory.read(index_low);
                        let high = memory.read(index_high);

                        let palette = if test_bit!(attributes, 5) {
                            memory.read(OBP0)
                        } else {
                            memory.read(OBP1)
                        };

                        let obj_pixels = PPU::mix_bytes_obj(low, high, y_flip, x_flip);
                        for p in 0..8 {
                            self.obj_fifo.push_back(Pixel::new(
                                obj_pixels[p],
                                palette,
                                0,
                                attributes & 0x80 >> 7,
                            ));
                        }
                        break;
                    }
                }
                // }
                //println!("{:?}", self.obj_fifo);

                self.internal_render(self.fx as usize);

                self.fx = self.fx + 1;
            }
            self.mode = 0;
        }

        if self.ly > 143 {
            memory.request_interrupt(VBLANK_I);
            self.mode = 1;
        }

        self.stat |= self.mode;
    }

    fn internal_render(&mut self, x: usize) {
        for i in 0..8 {
            let pixel = self.bg_fifo.pop_front().unwrap_or(Pixel::new(3, 0, 0, 0));
            let obj_pixel = self.obj_fifo.pop_front().unwrap_or(Pixel::new(0, 0, 0, 1));

            let index = (self.fx as usize * 8 + i + LCD_WIDTH * self.ly as usize) * 4;
            if index >= self.bg.len() {
                println!("index broken oops");
                break;
            }
            let mut bgp = [0u8; 4];
            bgp[0] = (pixel.palette) & 0b11;
            bgp[1] = (pixel.palette >> 2) & 0b11;
            bgp[2] = (pixel.palette >> 4) & 0b11;
            bgp[3] = (pixel.palette >> 6) & 0b11;

            let mut obp = [0u8; 4];
            obp[0] = (obj_pixel.palette) & 0b11;
            obp[1] = (obj_pixel.palette >> 2) & 0b11;
            obp[2] = (obj_pixel.palette >> 4) & 0b11;
            obp[3] = (obj_pixel.palette >> 6) & 0b11;

            let bg_color = PALETTE[bgp[pixel.color as usize] as usize];
            let obj_color = PALETTE_OBJ[obp[obj_pixel.color as usize] as usize];
            if obj_pixel.bg_prio == 1 || obj_pixel.color as usize == 0 {
                self.bg[index] = bg_color[3];
                self.bg[index + 1] = bg_color[2];
                self.bg[index + 2] = bg_color[1];
                self.bg[index + 3] = bg_color[0];
            } else {
                self.bg[index] = bg_color[3];
                self.bg[index + 1] = obj_color[2];
                self.bg[index + 2] = obj_color[1];
                self.bg[index + 3] = obj_color[0];
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        return self.ly == 153;
    }

    fn check_lcdc(&self, mask: u8) -> bool {
        self.lcdc & mask != 0
    }

    pub fn render(&mut self, texture: &mut Texture) -> Result<(), String> {
        texture.update(None, &mut self.bg, LCD_WIDTH * 4).unwrap();
        Ok(())
    }
}
