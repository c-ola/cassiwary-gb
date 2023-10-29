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
    
    // Interrupt
    INTn(u16),

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

use Instruction::*;

impl Instruction  {
    pub fn decode(opcode: u8) -> Instruction {
        let op_x = opcode >> 6;
        let op_y = (opcode & 0b00111000) >> 3;
        let op_z = opcode & 0b00000111;
        let op_p = op_y >> 1;
        let cc = op_y & 0b011;
        let op_q = op_y % 2;

        return if opcode == 0x00 {
            NOP
        } else if opcode == 0x10 {
            STOP
        } else if opcode == 0x76 {
            HALT
        } else if opcode == 0xF3 {
            DI
        } else if opcode == 0xFB {
            EI
        }
        // 8-bit load / store instructions registers
        // LD r r
        else if op_x == 0b01 {
            LDRwR(op_y, op_z)
        }
        // LD r n
        else if op_x == 0b00 && op_z == 0b110{
            LDRwN(op_y)
        }
        // LDH
        // 0xE0, 0xE2, 0xF0, 0xF2
        // 0b1110_0000
        // 0b1110_0010
        // 0b1111_0000
        // 0b1111_0010
        else if op_x == 0b11 && (op_y  == 0b110 || op_y == 0b100) && (op_z == 0b000 || op_z == 0b010) {
            LDH(op_y == 0b100, op_z == 0b000)
        }
        // LD A (nn)
        else if op_x == 0b11 && op_y & 0b101 == 0b101 && op_z == 0b010 {
            LDAwNNa(op_y == 0b101)
        }
        // LD (rr) A, LD A (rr)
        else if op_x == 0b00 && op_z == 0b010{
            if op_q == 0b1 {
                LDAwRRa(op_p)
            }
            else {
                LDRRawA(op_p)
            }
        }

        // 16-bit loads
        // LD rr nn
        else if op_x == 0b00 && op_q == 0b0 && op_z == 0b001 {
            LDrrnn(op_p)
        }
        //LD (nn) SP
        else if opcode == 0x08 {
            LDNNawSP
        }
        // load stack pointer from HL
        else if opcode == 0xF9 {
            LDSPwHL
        } 
        else if opcode == 0xF8 {
            LDHLwSP
        }
        // push rr
        else if op_x == 0b11 && op_q == 0 && op_z == 0b101{
            PUSHrr(op_p)
        }
        // pop rr
        else if op_x == 0b11 && op_q == 0 && op_z == 0b001{
            POPrr(op_p)
        }

        //-----------ARITHMETIC---------        

        //add, sub, adc, subc, and, or, xor, cp 
        else if op_x == 0b10 && op_q == 0b0 {
            match op_p {
                0 => Add(op_z),
                1 => Sub(op_z),
                2 => And(op_z),
                3 => Or(op_z),
                _ => ErrInstr{opcode},
            }
        }
        else if op_x == 0b10 && op_q == 0b1 {
            match op_p {
                0 => Adc(op_z),
                1 => Sbc(op_z),
                2 => Xor(op_z),
                3 => Cmp(op_z),
                _ => ErrInstr{opcode},
            }
        }
        // arithmetic with n
        else if op_x == 0b11 && op_z == 0b110 {
            match op_y {
                0b000 => Addn,
                0b001 => Adcn,
                0b010 => Subn,
                0b011 => Sbcn,
                0b100 => Andn,
                0b101 => Xorn,
                0b110 => Orn,
                0b111 => Cmpn,
                _ => ErrInstr{opcode},
            }
        } 
        // increment / decrement registers
        else if op_x == 0b00 && op_z == 0b100 {
            IncR(op_y)
        }
        else if op_x == 0b00 && op_z == 0b101 {
            DecR(op_y)
        }

        // 16-bit arithmetic
        // ADD SP e
        else if opcode == 0xE8 {
            AddSpE
        }
        // ADD HL rr
        else if op_x == 0b00 && op_q == 0b1 && op_z == 0b001 {
            ADDHLrr(op_p)
        }
        // INC / DEC
        else if op_x == 0b00 && op_q == 0b0 && op_z == 0b011 {
            INCrr(op_p)
        }
        else if op_x == 0b00 && op_q == 0b1 && op_z == 0b011 {
            DECrr(op_p)
        }
        else if opcode == 0x27 {
            DAA
        }
        else if opcode == 0x37 {
            SCF
        }
        else if opcode == 0x2F {
            CPL
        }
        else if opcode == 0x3F {
            CCF
        }       

        // ROTATES AND SHIFTS & Bit Ops
        // RLCA
        else if opcode == 0x07 {
            RLCA
        }
        // RLA
        else if opcode == 0x17 {
            RLA
        }
        // RRCA
        else if opcode == 0x0F {
            RRCA
        }
        // RRA
        else if opcode == 0x1F {
            RRA
        }
        // prefix CB
        else if opcode == 0xCB {
            CB
        }

        //--------CONTROL FLOW---------

        //Jump nn
        else if opcode == 0b1100_0011 {
            JPnn
        }
        //jump HL
        else if opcode == 0b1110_1001 {
            JPHL
        }
        //jump cc, nn
        else if op_x == 0b11 && op_y & 0b100 == 0b000 && op_z == 0b010 {
            JPccnn(cc)
        }
        // JR e
        else if opcode == 0x18 {
            JRe
        }

        //JR cc, e
        else if op_x == 0b00 && op_y & 0b100 == 0b100 && op_z == 0b000 {
            JRcce(cc)
        }
        // CALL nn
        else if opcode == 0xCD {
            CALLnn
        }
        //CALL cc, nn
        else if op_x == 0b11 && op_y & 0b100 == 0b000 && op_z == 0b100 {
            CALLccnn(cc)
        }
        //RET
        else if opcode == 0xC9 {
            RET
        }
        //RET cc
        else if op_x == 0b11 && op_y & 0b100 == 0b000 && op_z == 0b000 {
            RETcc(cc)
        }
        //RETI
        else if opcode == 0xD9 {
            RETI
        }
        //RST n
        else if op_x == 0b11 && op_z == 0b111 {
            RSTn(op_y)
        }
        else {
            ErrInstr{opcode}
        }
    }

    /// Decode CB prefix instructions
    pub fn decode_cb(opcode: u8) -> Instruction {
        let op_x = opcode >> 6;
        let op_y = (opcode & 0b00111000) >> 3;
        let op_z = opcode & 0b111;

        return match op_x {
            //these are all the rotates
            0b00 => {
                match op_y {
                    0b000 => RLCr(op_z),
                    0b001 => RRCr(op_z),
                    0b010 => RLr(op_z),
                    0b011 => RRr(op_z),
                    0b100 => SLAr(op_z),
                    0b101 => SRAr(op_z),
                    0b110 => SWAPr(op_z),
                    0b111 => SRLr(op_z),
                    _ => ErrInstr{opcode}
                }
            },

            //rest are bit instructions
            0b01 => BITnr{n: op_y, r: op_z},
            0b10 => RESnr{n: op_y, r: op_z},
            0b11 => SETnr{n: op_y, r: op_z},
            _ => ErrInstr{opcode},
        };
    }
}
