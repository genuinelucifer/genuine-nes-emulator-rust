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
            let instruction = self.ram.get_instruction(self.PC as usize);
            self.PC += 1;
            self.new_instruction(instruction);
            instruction
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
                            0x0 => {},
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
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            0x10 => {},
            0x20 => {
                match nibble & 0x0F {
                    0x01 => {
                        self.addressing_mode_indirect_x(&Self::instruction_and);
                    },
                    0x05 => {
                        // AND with accumulator zeropage 3 cycles, 2 bytes
                        self.addressing_mode_zero_page(&Self::instruction_and);
                    },
                    0x09 => {
                        // AND with accumulator #immediate 2 cycles, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_and);
                    },
                    0x0D => {
                        // AND with accumulator absolute 4 cycles, 3 bytes
                        self.addressing_mode_absolute(&Self::instruction_and);
                    },
                    _ => {}
                }
            },
            0x30 => {
                match nibble & 0x0F {
                    0x01 => {
                        self.addressing_mode_indirect_y(&Self::instruction_and);
                    },
                    0x05 => {
                        //AND with accumulator zeropage X 4 cycles, 2 bytes
                        self.addressing_mode_zero_page_with_index(true, &Self::instruction_and);
                    },
                    0x09 => {
                        //AND with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index(false, &Self::instruction_and);
                    },
                    0x0D => {
                        //AND with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index(true, &Self::instruction_and);
                    },
                    _ => {}
                }
            },
            0x40 => {},
            0x50 => {},
            0x60 => {
                match nibble & 0x0F {
                    0x01 => {
                        // ADC indirect X 6 cycle, 2 bytes
                        self.addressing_mode_indirect_x(&Self::instruction_adc);
                    },
                    0x05 => {
                        // ADC 3 cycle, 2 bytes
                        self.addressing_mode_zero_page(&Self::instruction_adc);
                    },
                    0x09 => {
                        // ADC immediate 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_adc);
                    },
                    0x0D => {
                        // ADC absolute 4 cycle, 3 bytes
                        self.addressing_mode_absolute(&Self::instruction_adc);
                    },
                    _ => {}
                }
            },
            0x70 => {
                match nibble & 0x0F {
                    0x01 => {
                        // ADC indirect Y 6 cycle, 2 bytes
                        self.addressing_mode_indirect_y(&Self::instruction_adc);
                    },
                    0x05 => {
                        // ADC 4 cycle, 2 bytes
                        self.addressing_mode_zero_page_with_index(true, &Self::instruction_adc);
                    },
                    0x08 => {
                        //SEI 2 cycle, 1 byte
                        match self.cycle {
                            0x00 => {
                                self.new_instruction(nibble);
                            },
                            0x01 => {
                                self.SR |= 0x4;
                                self.new_instruction = true;
                                self.cycle = 0;
                            },
                            _ => {}
                        }
                    },
                    0x09 => {
                        // ADC absolute Y 4* cycle, 3 bytes
                        self.addressing_mode_absolute_with_index(false, &Self::instruction_adc);
                    },
                    0x0D => {
                        // ADC absolute X 4* cycle, 3 bytes
                        self.addressing_mode_absolute_with_index(true, &Self::instruction_adc);
                    },
                    _ => {}
                }
            },
            0x80 => {
                match nibble & 0x0F {
                    0x01 => {
                        // STA indirect X 6 cycles, 2 bytes
                        self.addressing_mode_indirect_x(&Self::instruction_sta);
                    },
                    0x05 => {
                        // STA 3 cycle, 2 bytes
                        self.addressing_mode_zero_page(&Self::instruction_sta);
                    },
                    0x0A => {
                        // TXA 2 cycle, 1 byte
                        match self.cycle {
                            0x0 => {},
                            0x1 => {
                                self.AC = self.X;
                                self.SR |= self.AC & 0x80;
                                self.SR |= if self.AC == 0x0 {0x2} else {0x0};
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x0D => {
                        // STA absolute, 4 cycle, 3 bytes
                        self.addressing_mode_absolute(&Self::instruction_sta);
                    },
                    0x0E => {
                        // STX absolute, 4 cycle, 3 bytes
                        self.addressing_mode_absolute(&Self::instruction_stx);
                    }
                    _ => {
                    }
                }
            },
            0x90 => {
                match nibble & 0x0F {
                    0x01 => {
                        // STA indirect Y 6 cycles, 2 bytes
                        self.addressing_mode_indirect_y(&Self::instruction_sta);
                    },
                    0x05 => {
                        // STA X 4 cycle, 2 bytes
                        self.addressing_mode_zero_page_with_index(true, &Self::instruction_sta);
                    },
                    0x09 => {
                        //STA absolute Y, 5 cycle
                        self.addressing_mode_absolute_with_index(false, &Self::instruction_sta);
                    },
                    0x0A => {
                        // TXS 2 cycle, 1 byte
                        match self.cycle {
                            0x0 => {},
                            0x1 => {
                                self.SP = self.X;
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x0D => {
                        //STA absolute X, 5 cycle
                        self.addressing_mode_absolute_with_index(true, &Self::instruction_sta);
                    },
                    _ => {}
                }
            },
            0xA0 => {
                match nibble & 0x0F {
                    0x00 => {},
                    0x01 => {},
                    0x02 => {
                        // LDX 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_ldx);
                    },
                    0x03 => {},
                    0x04 => {},
                    0x05 => {},
                    0x06 => {},
                    0x07 => {},
                    0x08 => {},
                    0x09 => {
                        // LDA #$x immediate, 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_lda);
                    },
                    0x0A => {
                        // TAX transfer accumulator to index x (1byte)(2 cycle)
                        match self.cycle {
                            0x00 => {},
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
                            0x00 => {},
                            0x01 => {
                                self.X = ((self.X as u16 + 1u16)%0x100) as u8;
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

    fn new_instruction(&mut self, instruction: u8) {
        self.cycle = 1;
        self.new_instruction = false;
        self.current_instruction = instruction;
    }

    fn reset_instruction(&mut self) {
        self.new_instruction = true;
        self.cycle = 0;
    }

    // instructions
    // and with accumulator
    fn instruction_and(&mut self, byte: u8) {
        self.AC &= self.ram.get_instruction(byte as usize);
        self.set_flag_1st_bit_zero(self.AC);
        self.set_flag_7th_bit_nagetive(self.AC);
    }

    // add with carry
    fn instruction_adc(&mut self, byte: u8) {
        let sum:u16 = (self.AC as u16) + (byte as u16) + ((self.SR & 0x1) as u16);
        let sum_as_u8 = (sum%(0x100 as u16)) as u8;

        self.set_flag_0th_bit_carry(sum);
        self.set_flag_1st_bit_zero(sum_as_u8);
        self.set_flag_6th_bit_overflow(self.AC as u16, byte as u16, sum);
        self.set_flag_7th_bit_nagetive(sum_as_u8);
        self.AC = sum_as_u8;
    }

    // load into X
    fn instruction_ldx(&mut self, byte: u8) {
        self.X = byte;
        self.set_flag_1st_bit_zero(byte);
        self.set_flag_7th_bit_nagetive(byte);
    }

    // load into Accumulator
    fn instruction_lda(&mut self, byte: u8) {
        self.AC = byte;
        self.set_flag_1st_bit_zero(byte);
        self.set_flag_7th_bit_nagetive(byte);
    }

    // store accumulator in memory
    fn instruction_sta(&mut self, _: u8) {
        self.ram.set_address(self.AC, self.arg as usize);
    }

    // store X in memory {
    fn instruction_stx(&mut self, _: u8) {
        self.ram.set_address(self.X, self.arg as usize);
    }


    // set flags
    fn set_flag_0th_bit_carry(&mut self, result:u16) {
        self.SR |= if result > 0xff {0x1} else {0x0}; // carry flag 0th bit
    }

    fn set_flag_1st_bit_zero(&mut self, result:u8) {
        self.SR |= if result == 0x0 {0x2} else {0x0}; // zero flag 1st bit
    }

    fn set_flag_6th_bit_overflow(&mut self, x:u16, y:u16, z:u16) {
        // The overflow flag is set when the sign of the addends is the same and
        // differs from the sign of the sum
        // overflow = <'x' and 'y' have the same sign> &
        //           <the sign of 'x' and 'sum' differs> &
        //           <extract sign bit>
        self.SR |= if (!((x ^ z) & (x ^ y) & 0x80) as u8) == 0xFF {0x40} else {0x0}; // overflow flag 6th bit
    }

    fn set_flag_7th_bit_nagetive(&mut self, result: u8) {
        self.SR |= result & 0x80;
    }

    /**
     * addressing modes
     * function's name should have prefix `addressing_mode_`
     */
    fn addressing_mode_immediate(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                let byte = self.ram.get_instruction(self.PC as usize);
                self.PC += 1;
                instruction(self, byte);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_zero_page(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                instruction(self, self.arg as u8);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_zero_page_with_index(&mut self, is_x: bool, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.arg += (if is_x {self.X} else {self.Y}) as u16;
                self.cycle += 1;
            },
            0x3 => {
                let byte = self.ram.get_instruction(self.arg as usize);
                instruction(self, byte);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_absolute(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.arg |= (self.ram.get_instruction(self.PC as usize) as u16) << 8;
                self.PC += 1;
                self.cycle += 1;
            },
            0x3 => {
                let operand = self.ram.get_instruction(self.arg as usize);
                instruction(self, operand);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_absolute_with_index(&mut self, is_x:bool, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.arg |= (self.ram.get_instruction(self.PC as usize) as u16)<<8;
                self.PC += 1;
                self.cycle += 1;
            },
            0x3 => {
                self.arg += (if is_x {self.X} else {self.Y}) as u16;
                self.cycle += 1;
            },
            0x4 => {
                let operand = self.ram.get_instruction(self.arg as usize);
                instruction(self, operand);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_indirect_x(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.arg += self.X as u16;
                self.cycle += 1;
            },
            0x3 => {
                self.cycle += 1;
            },
            0x4 => {
                self.arg = ((self.ram.get_instruction(self.arg as usize) as u16) | (self.ram.get_instruction((self.arg+1) as usize) as u16)<<8) as u16;
                self.cycle += 1;
            },
            0x5 => {
                let operand = self.ram.get_instruction(self.arg as usize);
                instruction(self, operand);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_indirect_y(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {},
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.cycle += 1;
            },
            0x3 => {
                self.arg = ((self.ram.get_instruction(self.arg as usize) as u16) | (self.ram.get_instruction((self.arg+1) as usize) as u16)<<8) as u16;
                self.cycle += 1;
            },
            0x4 => {
                self.arg += self.Y as u16;
                self.cycle += 1;
            },
            0x5 => {
                let operand = self.ram.get_instruction(self.arg as usize);
                instruction(self, operand);
                self.reset_instruction();
            },
            _ => {}
        }
    }
}