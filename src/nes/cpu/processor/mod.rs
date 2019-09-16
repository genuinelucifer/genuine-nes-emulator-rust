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

pub mod memory;

#[allow(non_snake_case)]
pub struct Processor {
    PC: u16,
    AC: u8,
    X: u8,
    Y: u8,
    SR: u8,
    SP: u8,
    //stack: [u8; 0xFF] // ?
    ram: memory::Memory
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
            ram: memory
        }
    }

    pub fn execute_next_instruction(&mut self) {
        let nibble = self.ram.get_instruction(self.PC as usize);
        self.PC += 1;
        println!("inst1 :: {:#04X?}",nibble);
        let low_nibble = self.ram.get_instruction(self.PC as usize);
        self.PC += 1;

        match nibble & 0xF0 {
            0x00 => {
            },
            0x10 => {
            },
            0x20 => {
            },
            0x30 => {
            },
            0x40 => {
            },
            0x50 => {
            },
            0x60 => {
            },
            0x70 => {
            },
            0x80 => {
            },
            0x90 => {
            },
            0xA0 => {
                match nibble & 0x0F {
                    0x00 => {
                    },
                    0x01 => {
                    },
                    0x02 => {
                    },
                    0x03 => {
                    },
                    0x04 => {
                    },
                    0x05 => {
                    },
                    0x06 => {
                    },
                    0x07 => {
                    },
                    0x08 => {
                    },
                    0x09 => {

                    },
                    0x0A => {
                    },
                    0x0B => {
                    },
                    0x0C => {
                    },
                    0x0D => {
                    },
                    0x0E => {
                    },
                    0x0F => {
                    },
                    _ => {
                    },


                }
            },
            0xB0 => {
            },
            0xC0 => {
            },
            0xD0 => {
            },
            0xE0 => {
            },
            0xF0 => {
            },
            _ => {
            }
        }

    }
}