//Registers:
//
//PC	....	program counter	(16 bit)
//AC	....	accumulator	(8 bit)
//X	....	X register	(8 bit)
//Y	....	Y register	(8 bit)
//SR	....	status register [NV-BDIZC]	(8 bit)
//SP	....	stack pointer	(8 bit)
//
//SR Flags (bit 7 to bit 0):
//
//N	....	Negative
//V	....	Overflow
//-	....	ignored
//B	....	Break
//D	....	Decimal (use BCD for arithmetics)
//I	....	Interrupt (IRQ disable)
//Z	....	Zero
//C	....	Carry
//
//
//Processor Stack:
//LIFO, top down, 8 bit range, 0x0100 - 0x01FF
//
//
//Bytes, Words, Addressing:
//8 bit bytes, 16 bit words in lobyte-hibyte representation (Little-Endian).
//16 bit address range, operands follow instruction codes.
//
//Signed values are two's complement, sign in bit 7 (most significant bit).
//(%11111111 = $FF = -1, %10000000 = $80 = -128, %01111111 = $7F = +127)

// refer http://www.atarihq.com/danb/files/64doc.txt for cycle
// vflag http://www.6502.org/tutorials/vflag.html

pub mod memory;

#[allow(non_snake_case)]
pub struct Processor {
    PC: u16,
    AC: u8,
    X: u8,
    Y: u8,
    SR: u8,
    SP: u8,
    ram: memory::Memory,
    new_instruction: bool,
    current_instruction: u8,
    cycle: usize,
    arg: u16 // useful in 3bytes opcodes
}

#[allow(non_camel_case_types)]
enum AddressingMode {
    IMMEDIATE,
    IMPLIED,
    ACCUMULATOR,
    ABSOLUTE,
    ABSOLUTE_X,
    ABSOLUTE_Y,
    ZERO_PAGE,
    ZERO_PAGE_X,
    ZERO_PAGE_Y,
    RELATIVE,
    INDIRECT,
    INDEXED_INDIRECT,
    INDIRECT_INDEXED
}

impl Processor {
    pub fn new(data: &Vec<u8>) -> Processor {
        let mut memory = memory::Memory::new();
        memory.load(data, 0x8000);
        Processor {
            PC: 0x8000,
            AC: 0x00,
            X: 0x00,
            Y: 0x00,
            SR: 0x30,
            SP: 0xFF, //top down stack pointer from 0x0100 - 0x01FF
            ram: memory,
            new_instruction: true,
            current_instruction: 0x00,
            cycle: 0x00,
            arg: 0x00
        }
    }

    pub fn execute_next_instruction(&mut self) {
        let nibble = if self.new_instruction {
            self.ram.get_instruction(self.PC as usize)
        } else {
            self.current_instruction
        };

        println!("inst1 :: nibble: {:#04X?}, new_inst: {},  cur_inst: {:#04X}, cycle: {}, arg: {}",nibble, self.new_instruction, self.current_instruction, self.cycle, self.arg);
        println!("before registers AC: {:#X?}, X: {:#X?}, Y: {:#X?}, SP: {:#X?}, PC: {:?}, SR: {:08b}", self.AC, self.X, self.Y, self.SP, self.PC, self.SR);

        match nibble & 0xF0 {
            0x00 => {
                match nibble & 0x0F {
                    0x00 => {
                        //BRK 7 cycle, 1byte
                        match self.cycle {
                            0x0 => {
                                self.PC += 1;
                                self.cycle += 1;
                                self.current_instruction = nibble;
                                self.new_instruction = false;
                            },
                            0x1 => {
                                self.PC += 1;
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.ram.set_address((self.PC >> 8) as u8, self.SP as usize);
                                self.SP -= 1;
                                self.cycle += 1;
                            },
                            0x3 => {
                                self.ram.set_address((self.PC & 0xFF) as u8, self.SP as usize);
                                self.SP -= 1;
                                self.cycle += 1;
                            },
                            0x4 => {
                                self.ram.set_address((self.SR | 0x10) as u8, self.SP as usize);
                                self.SP -= 1;
                                self.cycle += 1;
                            },
                            0x5 => {
                                self.PC |= ((self.ram.get_instruction(0xFFFE) as u16) << 8) as u16;
                                self.cycle += 1;
                            },
                            0x6 => {
                                self.PC |= self.ram.get_instruction(0xFFFF) as u16;
                                self.SR |= 0x10; // set B flag
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            0x10 => {},
            0x20 => {},
            0x30 => {},
            0x40 => {},
            0x50 => {},
            0x60 => {
                match nibble & 0x0F {
                    0x09 => {
                        // ADC immediate 2 cycle, 2 bytes
                        match self.cycle {
                            0x00 => {
                                //read opcode
                                self.PC += 1;
                                self.cycle += 1;
                                self.new_instruction = false;
                                self.current_instruction = nibble;
                            },
                            0x01 => {
                                let operand = self.ram.get_instruction(self.PC as usize);
                                self.PC += 1;
                                let sum:u16 = (self.AC as u16) + (operand as u16) + ((self.SR & 0x1) as u16);
                                println!("sum: {}",sum);
                                let sum_as_i8 = (sum%(0x100 as u16)) as u8;
                                println!("sumu8: {}", sum_as_i8);

                                self.SR |= if sum > 0xff {0x1} else {0x0}; // carry flag 0th bit
                                // The overflow flag is set when the sign of the addends is the same and
                                // differs from the sign of the sum
                                // overflow = <'AC' and 'operant' have the same sign> &
                                //           <the sign of 'AC' and 'sum' differs> &
                                //           <extract sign bit>
                                self.SR |= if (!((self.AC as u16^ sum) & (self.AC as u16 ^ operand as u16) & 0x80) as u8) == 0xFF {0x40} else {0x0}; // overflow flag 6th bit
                                self.SR |= sum_as_i8 & 0x80; //check 7th bit for negative result
                                self.SR |= if sum_as_i8 == 0x0 {0x2} else {0x0}; // zero flag 1st bit
                                self.AC = sum_as_i8;
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {
                                //unreachable
                            }
                        }
                    },
                    _ => {}
                }
            },
            0x70 => {
                match nibble & 0x0F {
                    0x08 => {
                        //SEI 2 cycle, 1 byte
                        match self.cycle {
                            0x00 => {
                                self.PC += 1;
                                self.current_instruction = nibble;
                                self.new_instruction = false;
                                self.cycle += 1;
                            },
                            0x01 => {
                                self.SR |= 0x4;
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            0x80 => {
                match nibble & 0x0F {
                    0x0D => {
                        //STA absolute, 4 cycle
                        match self.cycle {
                            0x00 => {
                                //read opcode
                                self.PC += 1;
                                self.cycle += 1;
                                self.new_instruction = false;
                                self.current_instruction = nibble;
                            },
                            0x01 => {
                                //read operand
                                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                                self.PC += 1;
                                self.cycle += 1;
                            },
                            0x02 => {
                                self.arg = self.arg << 8 | self.ram.get_instruction(self.PC as usize) as u16;
                                self.PC += 1;
                                self.cycle += 1;
                            },
                            0x03 => {
                                self.ram.set_address(self.AC, self.arg as usize);
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {

                            }
                        }
                    },
                    _ => {
                    }
                }
            },
            0x90 => {},
            0xA0 => {
                match nibble & 0x0F {
                    0x00 => {},
                    0x01 => {},
                    0x02 => {
                        // LDX 2 cycle, 2 bytes
                        match self.cycle {
                            0x0 => {
                                self.PC += 1;
                                self.new_instruction = false;
                                self.cycle += 1;
                                self.current_instruction = nibble;
                            },
                            0x1 => {
                                let data = self.ram.get_instruction(self.PC as usize);
                                self.PC += 1;
                                self.X = data;
                                self.SR |= data & 0x80;
                                self.SR |= if data == 0x0 {0x2} else {0x0};
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {}
                        }
                    },
                    0x03 => {},
                    0x04 => {},
                    0x05 => {},
                    0x06 => {},
                    0x07 => {},
                    0x08 => {},
                    0x09 => {
                        // LDA #$x immediate, 2 cycle, 2 bytes
                        match self.cycle {
                            0x00 => {
                                //read opcode
                                self.PC += 1;
                                self.cycle += 1;
                                self.new_instruction = false;
                                self.current_instruction = nibble;
                            },
                            0x01 => {
                                //read immediate value and load into AC
                                self.AC = self.ram.get_instruction(self.PC as usize);
                                self.PC += 1;
                                self.SR |= self.AC & 0x80; //check 7th bit for negative result
                                self.SR |= if self.AC == 0x0 {0x2} else {0x0};
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {
                                //unreachable
                            }
                        }
                    },
                    0x0A => {
                        // TAX transfer accumulator to index x (1byte)(2 cycle)
                        match self.cycle {
                            0x00 => {
                                self.PC += 1;
                                self.cycle += 1;
                                self.new_instruction = false;
                                self.current_instruction = nibble;
                            },
                            0x01 => {
                                self.X = self.AC;
                                self.SR |= self.X & 0x80;
                                self.SR |= if self.X == 0x0 {0x2} else {0x0};
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {
                                //unreachable
                            }
                        }
                    },
                    0x0B => {},
                    0x0C => {},
                    0x0D => {},
                    0x0E => {},
                    0x0F => {},
                    _ => {},
                }
            },
            0xB0 => {},
            0xC0 => {},
            0xD0 => {},
            0xE0 => {
                match nibble & 0x0F {
                    0x08 => {
                        // INX increment X (2cycle, 1 byte)
                        match self.cycle {
                            0x00 => {
                                self.PC += 1;
                                self.cycle += 1;
                                self.new_instruction = false;
                                self.current_instruction = nibble;
                            },
                            0x01 => {
                                self.X += 1;
                                self.SR |= self.X & 0x80;
                                self.SR |= if self.X == 0x0 {0x2} else {0x0};
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            0xF0 => {},
            _ => {}
        }

        println!("after registers AC: {:#X?}, X: {:#X?}, Y: {:#X?}, SP: {:#X?}, PC: {:?}, SR: {:08b}", self.AC, self.X, self.Y, self.SP, self.PC, self.SR);
        println!();
    }
}