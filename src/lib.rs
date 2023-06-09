pub mod console;
pub mod bytes;

use crate::console::*;

#[cfg(test)]
mod tests {
    use crate::*;
 
    #[test]
    fn launch() {
        let mut gb = GameBoy::new();
        gb.init();
        gb.gamepack.write(0, 0x00);
        gb.set_boot(false);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(B), 0x00);
        assert_eq!(gb.peek_memory().read(0x0000), 0x00);   
    }
    
    #[test]
    fn loads() {
        let mut gb = GameBoy::new();
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
        let mut gb = GameBoy::new();
        gb.init();
        gb.set_run_count(3);
        
        //load a 5
        gb.write(0b0011_1110);
        gb.write(0b0000_0101); 

        //load b 3
        gb.write(0b0000_0110);
        gb.write(0x03);

        //load de 0x03A1
        gb.write(0x11);
        gb.write(0x03);
        gb.write(0xA1);

        gb.start();

        //add b
        gb.inc_instr_count();
        gb.write(0b10000000);
        gb.start();
        assert_eq!(gb.peek_cpu().get_reg(A), 0x08);

    }
}
