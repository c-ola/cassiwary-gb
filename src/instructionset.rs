const LDREG: u8 = 0b01000000;

fn decode(opcode: u16){
    let bytes = opcode.to_be_bytes(); 
    let high = bytes[0];
    let low = bytes[1];
    
    if high & LDREG == 0 {
        let x = 
    }
}
