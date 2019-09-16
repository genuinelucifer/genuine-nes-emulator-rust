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
            SR: 0x00,
            SP: 0xFF, //top down stack pointer from 0x0100 - 0x01FF
            ram: memory
        }
    }

    pub fn execute_next_instruction(&self) {
        let instruction = self.ram.get_instruction(self.PC as usize);
        println!("inst1 :: {:#x?}",instruction);
    }
}