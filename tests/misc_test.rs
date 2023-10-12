use cassowary_gb::console::*;
use cassowary_gb::console::regids::*;

#[test]
pub fn misc() {
    let mut gb = GameBoy::new();
    gb.init();
    let data = [
        0x00,
        0x10,
        0xE1
    ];
    gb.load_memory(&data);
    gb.run_n(3);

    gb.log_memory();
    assert!(false);
}


