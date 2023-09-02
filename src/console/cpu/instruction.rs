pub enum Instruction {  
    /*
     * 8-bit Load
     */
    LDrWithrn(u8, u8),
    LDAddrHLWithrn(u8, u8),
    LDABC,



    /*
     * 16-bit Load
     */
    LDrrnn(u8, u16),
    LDAddrnnSP(u16),
    LDSPHL,
    PUSHrr(u8),
    POPrr(u8),

    /*
     * 8-bit Arithmetic/Logic
     */


    /*
     * 16-bit Arithmetic/Logic
     */
    ADDHLrr(u8),
    INCrr(u8),
    DECrr(u8),
    ADDSPdd(i8),
    LDHLSPdd(i8), // this one is a load but it uses the ALU

    /*
     * Rotates and Shifts
     */
    RLCA,
    RLA,
    RRCA,
    RRA,
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
    BITnr(u8), //same as BIT n, (HL)
    SETnr(u8), //same as SET n, (HL)
    RESnr(u8), //same as RESET n, (HL)

    /*
     * CPU Control
     */

    CCF,
    SCF,
    NOP,
    HALT,
    STOP,
    DI,
    EI,

    /*
     * Jumps
     */
    JPnn(u16), // same as JP HL, just pass in HL
    JPfnn(u8, u16),
    JRPCdd(i8),
    JRfPCdd(i8),
    CALLnn(u16),
    CALLfnn(u8, u16),
    RET,
    RETf(u8),
    RETI,
    RSTn(u8),
}

impl Instruction {

}

