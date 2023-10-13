//for explicitness
#[derive(Debug, Copy, Clone)]
pub enum Instruction {  
    /*
     * 8-bit Load
     */
    LDRwN(u8), // indirect HL included here
    LDRwR(u8, u8),
    LDH(bool, bool), //Load from accumulator to memory if true, use n instead of c if true,  
    LDAwNNa(bool), // load from if true
    
    LDRRawA(u8),
    LDAwRRa(u8),

    /*
    * 16-bit Load
     */
    LDrrnn(u8),
    LDNNawSP,
    PUSHrr(u8),
    POPrr(u8),
    LDHLwSP,
    LDSPwHL,

    /*
     * 8-bit Arithmetic/Logic
     */
    Add(u8),
    Adc(u8),
    Sub(u8),
    Sbc(u8),
    And(u8),
    Or(u8),
    Xor(u8),
    Cmp(u8),
    IncR(u8),
    DecR(u8),
    Addn,
    Subn,
    Andn,
    Orn,
    Adcn,
    Sbcn,
    Xorn,
    Cmpn,

    /*
     * Arithmetic Miscellaneous
     */
    DAA,
    SCF,
    CCF,
    CPL,

    /*
     * 16-bit Arithmetic/Logic
     */
    ADDHLrr(u8),
    AddSpE,
    INCrr(u8),
    DECrr(u8),

    /*
     * Rotates and Shifts
     */
    RLCA,
    RLA,
    RRCA,
    RRA,
    CB,
    RLCr(u8), //same with HL
    RLr(u8),
    RRCr(u8),
    RRr(u8),
    SLAr(u8),
    SWAPr(u8),
    SRAr(u8),
    SRLr(u8),

    /*
     * Single bit Operations
     */
    BITnr{n: u8, r: u8}, //same as BIT n, (HL)
    SETnr{n: u8, r: u8}, //same as SET n, (HL)
    RESnr{n: u8, r: u8}, //same as RESET n, (HL)

    /*
     * CPU Misc
     */

    NOP,
    HALT,
    STOP,
    DI,
    EI,

    /*
     * Jumps
     */
    JPnn,
    JPHL,
    JPccnn(u8),
    JRe,
    JRcce(u8),
    CALLnn,
    CALLccnn(u8),
    RET,
    RETcc(u8),
    RETI,
    RSTn(u8),

    ErrInstr{opcode: u8}
}
