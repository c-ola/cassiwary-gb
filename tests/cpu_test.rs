use cassowary_gb::console::*;
use cassowary_gb::console::regids::*;

#[test]
fn launch() {
    let mut gb = GameBoy::new();
    //gb.init();
    //gb.set_run_count(0);
    //assert_eq!(gb.peek_cpu().get_reg(B), 0x00);
    //assert_eq!(gb.peek_memory().read(0x0000), 0x00);   
}

#[test]
fn control(){
    let mut gb = GameBoy::new();
    gb.init();

    let data = [
        0xC3, 0x07, 0x00, // JP nn
        0xF3, // DI
        0xFB, // EI
        0x18, 0x03, // JR e
        0xCA, 0x03, 0x00, // JP cc, nn
        0x31, 0x00, 0xF0, // LD SP nn
        0xCD, 0x00, 0x01, // call 0x0100
        0x00,             // NOP
        0x0F1, // pop AF
        0xCC, 0x02, 0x01, // call c, nn
        0x00, // nop
        0b11_111_111, // rst 0b110
    ];
    gb.load_memory(&data);

    gb.gamepack.write(0x0100, 0x00); //nop
    gb.gamepack.write(0x0101, 0xC9); //ret

    gb.gamepack.write(0x0102, 0x00); //nop
    gb.gamepack.write(0x0103, 0xC9); // reti

    gb.gamepack.write(0x0100, 0x00);

    //step through instructions here
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().pc, 0x07); // jp nn

    gb.run_n(1);
    assert_eq!(gb.peek_cpu().pc, 0x03); // jp cc nn
                                        //
    gb.run_n(3);
    assert_eq!(gb.peek_cpu().pc, 0x0A); // jr e

    gb.run_n(2);
    assert_eq!(gb.peek_cpu().pc, 0x0100); // call 0x0100

    gb.run_n(2);
    assert_eq!(gb.peek_cpu().pc, 0x10); // ret addr

    gb.run_n(2);
    assert_eq!(gb.peek_cpu().get_reg(A), 0);

    gb.run_n(3);
    assert_eq!(gb.peek_cpu().pc, 21); // reti addr

    gb.run_n(2);
    assert_eq!(gb.peek_cpu().pc, 0x3800);   // rst 3 return addr
}

#[test]
fn loads() {
    let mut gb = GameBoy::new();
    gb.init();

    //load A, 5
    gb.write(0b0011_1110);
    gb.write(0b0000_0101);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 0x05);

    //load B, 2
    gb.write(0b0000_0110);
    gb.write(0x03);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(B), 0x03);

    //load C, B
    gb.write(0x48);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(B), gb.peek_cpu().get_reg(C));

    //LD DE, 0x03A1
    gb.write(0x11);
    gb.write(0x03);
    gb.write(0xA1);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_rr(DE), 0xA103);

    //load HL, 0xFB02
    gb.write(0b0010_0001);
    gb.write(0xFF);
    gb.write(0x02);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_rr(HL), 0x02FF);

    //LD (HL+), A
    gb.write(0x22);
    gb.run_n(1);
    let hl = gb.peek_cpu().get_rr(HL);
    //gb.gamepack.print(0x00FF, 1);
    assert_eq!(hl, 0x0300);
    assert_eq!(gb.peek_memory().read(hl - 1), gb.peek_cpu().get_reg(A));

    //LD (HL), 0xE3
    gb.write(0b0011_0110);
    gb.write(0xE3);
    gb.run_n(1);
    assert_eq!(gb.peek_memory().read(gb.peek_cpu().get_rr(HL)), 0xE3);

    //LDH A, (C)
    gb.write(0xF2);
    gb.run_n(1);
    assert_eq!(gb.peek_memory().read(0xFF03), gb.peek_cpu().get_reg(A));

    //LD SP, HL
    gb.write(0xF9);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_rr(HL), gb.peek_cpu().get_rr(SP));

    //PUSH rr
    gb.write(0b1101_0101);
    gb.run_n(1);
    assert_eq!((gb.peek_memory().read(0x0003), gb.peek_memory().read(0x02FF)), (0x03, 0xA1));

    //POP rr
    gb.write(0b1100_0001);
    gb.run_n(1);
    //gb.gamepack.print(0xFF00, 1);
    assert_eq!(gb.peek_cpu().get_rr(BC) ,0xA103);
}

#[test]
fn prefix_instructions() {
    let mut gb = GameBoy::new();
    gb.init();

    gb.write(0b0011_1110);
}


#[test]
fn arithmetic(){
    let mut gb = GameBoy::new();
    gb.init();

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

    gb.run_n(5);

    //add b
    gb.write(0x80);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 0x137u16 as u8);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);

    //daa
    gb.write(0x27);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);


    gb.write(0b0011_1110);
    gb.write(0x37); 
    gb.run_n(1);


    //sub c
    gb.write(0x91);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 125);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0111_0000);

    // and e
    gb.write(0xA3);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 0x1);

    // or d
    gb.write(0xB2);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 161);

    // xor e
    gb.write(0xAB);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_reg(A), 162);

    // cp H
    gb.write(0xBC);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0100_0000);

    //scf
    gb.write(0x37);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0001_0000);

    //ccf
    gb.write(0x3F);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_flag(), 0b0000_0000);

    //cpl
    let a = gb.peek_cpu().get_reg(A);
    gb.write(0x2F);
    gb.run_n(1);
    assert_eq!(!a, gb.peek_cpu().get_reg(A));
    assert_eq!(gb.peek_cpu().get_flag(), 0b0110_0000);

    //add HL, BC
    gb.write(0x09);
    gb.run_n(1);
    assert_eq!(gb.peek_cpu().get_rr(HL), (0xB3BA + 0x103e) as u16);


    //havent tested INC instructions or ADC or rotations
}
