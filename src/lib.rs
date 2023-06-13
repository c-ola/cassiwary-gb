pub mod console;
pub mod bytes;
use crate::console::*;

//use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests {
    use crate::ppu::{Tile, PPU};
    use crate::console::*;


#[test]
    fn tiles() {

        let data = [
            0x3C, 0x7E,
            0x42, 0x42,
            0x42, 0x42,
            0x42, 0x42,
            0x7E, 0x5E,
            0x7E, 0x0A,
            0x7C, 0x56,
            0x38, 0x7C,
        ];

        let tile = Tile::new(data);
        //tile.render();

        //panic!();
    }

#[test]
    fn tilemap(){
        //let tilemap = PPU::new();
        //tilemap.render();
        // panic!();
    }

#[test]
    fn launch() {
        let mut gb = GameBoy::new_test();
        gb.init();
        gb.gamepack.write(0, 0x00);
        gb.set_boot(false);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(B), 0x00);
        assert_eq!(gb.peek_memory().read(0x0000), 0x00);   
    }

#[test]
    fn loads() {
        let mut gb = GameBoy::new_test();
        gb.init();
        gb.set_run_count(0);

        //load A, 5
        gb.inc_instr_count();
        gb.write(0b0011_1110);
        gb.write(0b0000_0101);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x05);

        //load B, 2
        gb.inc_instr_count();
        gb.write(0b0000_0110);
        gb.write(0x03);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(B), 0x03);

        //load C, B
        gb.inc_instr_count();
        gb.write(0x48);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(B), gb.peek_cpu().get_reg(C));

        //LD DE, 0x03A1
        gb.inc_instr_count();
        gb.write(0x11);
        gb.write(0x03);
        gb.write(0xA1);
        gb.start();
        assert_eq!(gb.peek_cpu().get_rr(DE), 0x03A1);

        //load HL, 0xFB02
        gb.inc_instr_count();
        gb.write(0b0010_0001);
        gb.write(0xFF);
        gb.write(0x02);
        gb.start();
        assert_eq!(gb.peek_cpu().get_rr(HL), 0xFF02);

        //LD (HL+), A
        gb.inc_instr_count();
        gb.write(0x22);
        gb.start();
        let hl = gb.peek_cpu().get_rr(HL);
        gb.gamepack.print(0xFF00, 1);
        assert_eq!(hl, 0xFF03);
        assert_eq!(gb.peek_memory().read(hl - 1), gb.peek_cpu().get_reg(A));

        //LD (HL), 0xE3
        gb.inc_instr_count();
        gb.write(0b0011_0110);
        gb.write(0xE3);
        gb.start();
        assert_eq!(gb.peek_memory().read(gb.peek_cpu().get_rr(HL)), 0xE3);

        //LDH A, (C)
        gb.inc_instr_count();
        gb.write(0xF2);
        gb.start();
        gb.gamepack.print(0xFF00, 1);
        assert_eq!(gb.peek_memory().read(0xFF03), gb.peek_cpu().get_reg(A));

        //LD SP, HL
        gb.inc_instr_count();
        gb.write(0xF9);
        gb.start();
        assert_eq!(gb.peek_cpu().get_rr(HL), gb.peek_cpu().get_rr(SP));

        //PUSH rr
        gb.inc_instr_count();
        gb.write(0b1101_0101);
        gb.start();
        gb.gamepack.print(0xFF00, 1);
        assert_eq!((gb.peek_memory().read(0xFF01), gb.peek_memory().read(0xFF02)), (0x03, 0xA1));

        //POP rr
        gb.inc_instr_count();
        gb.write(0b1100_0001);
        gb.start();
        gb.gamepack.print(0xFF00, 1);
        assert_eq!(gb.peek_cpu().get_rr(BC) ,0x03A1);
    }


#[test]
    fn arithmetic(){
        let mut gb = GameBoy::new_test();
        gb.init();
        gb.set_run_count(5);

        //load a 84
        gb.write(0b0011_1110);
        gb.write(0b1000_0100); 

        //load b B3
        gb.write(0b0000_0110);
        gb.write(0b1011_0011);

        //load c BA
        gb.write(0b0000_1110);
        gb.write(0b1011_1010);

        //load HL 0x3E10
        gb.write(0x21);
        gb.write(0x3E);
        gb.write(0x10);

        //load de 0x03A1
        gb.write(0x11);
        gb.write(0b0000_0011);
        gb.write(0b1010_0001);

        gb.start();

        //add b
        gb.inc_instr_count();
        gb.write(0x80);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x137u16 as u8);
        assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);

        //daa
        gb.inc_instr_count();
        gb.write(0x27);
        gb.start();
        assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);


        gb.inc_instr_count();
        gb.write(0b0011_1110);
        gb.write(0x37); 
        gb.start();


        //sub c
        gb.inc_instr_count();
        gb.write(0x91);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x3E);
        assert_eq!(gb.peek_cpu().get_flag(), 0b0111_0000);

        // and e
        gb.inc_instr_count();
        gb.write(0xA3);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x20);

        // or d
        gb.inc_instr_count();
        gb.write(0xB2);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x23);

        // xor e
        gb.inc_instr_count();
        gb.write(0xAB);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x82);

        // cp H
        gb.inc_instr_count();
        gb.write(0xBC);
        gb.start();
        assert_eq!(gb.peek_cpu().get_flag(), 0b0110_0000);

        //scf
        gb.inc_instr_count();
        gb.write(0x37);
        gb.start();
        assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);

        //ccf
        gb.inc_instr_count();
        gb.write(0x3F);
        gb.start();
        assert_eq!(gb.peek_cpu().get_flag(), 0b0000_0000);

        //cpl
        gb.inc_instr_count();
        let a = gb.peek_cpu().get_reg(A);
        gb.write(0x2F);
        gb.start();
        assert_eq!(!a, gb.peek_cpu().get_reg(A));
        assert_eq!(gb.peek_cpu().get_flag(), 0b0110_0000);

        //add HL, BC
        gb.inc_instr_count();
        gb.write(0x09);
        gb.start();
        assert_eq!(gb.peek_cpu().get_rr(HL), 0xB3BA + 0x3E10);


        //havent tested INC instructions or ADC or rotations

    }

#[test]
    fn control(){
        let mut gb = GameBoy::new_test();
        gb.init();
        gb.set_run_count(0);

        //write out asm starting here
        //JP nn ; 0
        gb.write(0xC3);
        gb.write(0x00);
        gb.write(0x07);

        //DI ; 3
        gb.write(0xF3);

        //EI ; 4
        gb.write(0xFB);

        //JR e ; 5
        gb.write(0x18);
        gb.write(0x03);

        //JP cc, nn ; 7
        gb.write(0xC2);
        gb.write(0x00);
        gb.write(0x03); // jump to ; 3

        //LD SP nn ; 10
        gb.write(0x31);
        gb.write(0xf0);
        gb.write(0x00);

        //call 0x0100 ; 13
        gb.write(0xCD);
        gb.write(0x01);
        gb.write(0x00); //; 15

        gb.write(0x00); // nop

        gb.write(0xF1); // pop AF (sets a to 0 for conditional call)

        gb.write(0xCC); // call c , nn
        gb.write(0x01);
        gb.write(0x02);

        gb.write(0x00); // nop

        gb.write(0b11_111_111); // rst 0b110


        gb.gamepack.write(0x0100, 0x00); //nop
        gb.gamepack.write(0x0101, 0xC9); //ret

        gb.gamepack.write(0x0102, 0x00); //nop
        gb.gamepack.write(0x0103, 0xC9); // reti

        gb.gamepack.write(0x0100, 0x00);

        //step through instructions here
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x07); // jp nn

        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x03); // jp cc nn

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x0A); // jr e

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x0100); // call 0x0100

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x10); // ret addr

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0);

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 21); // reti addr

        gb.inc_instr_count();
        gb.inc_instr_count();
        gb.start();
        assert_eq!(gb.peek_cpu().pc, 0x3800);   // rst 3 return addr
    }
}
/*
   pub fn set_panic_hook() {
// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization, and then
// we will get better error messages if our code ever panics.
//
// For more details see
// https://github.com/rustwasm/console_error_panic_hook#readme
#[cfg(feature = "console_error_panic_hook")]
console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern {
pub fn alert(s: &str);
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn greet(name: &str) {
alert(&format!("Hello, {}!", name));
}*/
