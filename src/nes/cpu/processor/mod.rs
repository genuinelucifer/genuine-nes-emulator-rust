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
                            0x0 => {
                                self.cycle = 1;
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
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_or);
                    },
                    0x05 => {
                        // OR with accumulator zeropage 3 cycles, 2 bytes
                        self.addressing_mode_zero_page_read(&Self::instruction_or);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_read_write(&Self::instruction_asl_memory);
                    },
                    0x08 => {
                        // PHP Push Processor Status on Stack 3 cycles, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.ram.set_address(self.SR, self.SP as usize);
                                self.SP -= 1;
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x09 => {
                        // OR with accumulator #immediate 2 cycles, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_or);
                    },
                    0x0A => {
                        // ASL accumulator 2 cycles, 1 byte
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_asl_accumulator);
                    },
                    0x0D => {
                        // OR with accumulator absolute 4 cycles, 3 bytes
                        self.addressing_mode_absolute_read(&Self::instruction_or);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_read_write(&Self::instruction_asl_memory);
                    },
                    _ => {}
                }
            },
            0x10 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BPL change branch if N==0
                        self.addressing_mode_relative(&Self::instruction_bpl);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_or);
                    },
                    0x05 => {
                        //OR with accumulator zeropage X 4 cycles, 2 bytes
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_or);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_with_index_read_write(true, &Self::instruction_asl_memory);
                    },
                    0x08 => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_clc);
                    },
                    0x09 => {
                        //OR with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_or);
                    },
                    0x0D => {
                        //OR with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_or);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_with_index_read_write(true, &Self::instruction_asl_memory);
                    },
                    _ => {}
                }
            },
            0x20 => {
                match nibble & 0x0F {
                    0x00 => {
                        // JSR 6 cycles, 3 bytes
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                                self.PC += 1;
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.cycle += 1;
                            },
                            0x3 => {
                                self.ram.set_address((self.PC>>8) as u8, self.SP as usize );
                                self.SP -= 1;
                                self.cycle += 1;
                            },
                            0x4 => {
                                self.ram.set_address((self.PC & 0xFF) as u8, self.SP as usize );
                                self.SP -= 1;
                                self.cycle += 1;
                            },
                            0x5 => {
                                let high = (self.ram.get_instruction(self.PC as usize) as u16) << 8;
                                self.PC = high | self.arg;
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_and);
                    },
                    0x04 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_bit);
                    },
                    0x05 => {
                        // AND with accumulator zeropage 3 cycles, 2 bytes
                        self.addressing_mode_zero_page_read(&Self::instruction_and);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_write(&Self::instruction_rol_memory);
                    },
                    0x08 => {
                        // PLP 4 cycles, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.SP += 1;
                                self.cycle += 1;
                            },
                            0x3 => {
                                self.SR = self.ram.get_instruction(self.SP as usize);
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x09 => {
                        // AND with accumulator #immediate 2 cycles, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_and);
                    },
                    0x0A => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_rol_accumulator);
                    },
                    0x0C => {
                        self.addressing_mode_absolute_read(&Self::instruction_bit);
                    },
                    0x0D => {
                        // AND with accumulator absolute 4 cycles, 3 bytes
                        self.addressing_mode_absolute_read(&Self::instruction_and);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_write(&Self::instruction_rol_memory);
                    },
                    _ => {}
                }
            },
            0x30 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BME change branch if result was negative
                        self.addressing_mode_relative(&Self::instruction_bmi);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_and);
                    },
                    0x05 => {
                        //AND with accumulator zeropage X 4 cycles, 2 bytes
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_and);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_with_index_write(true, &Self::instruction_rol_memory);
                    },
                    0x09 => {
                        //AND with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_and);
                    },
                    0x0D => {
                        //AND with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_and);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_with_index_write(true, &Self::instruction_rol_memory);
                    },
                    _ => {}
                }
            },
            0x40 => {
                match nibble & 0x0F {
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_xor);
                    },
                    0x05 => {
                        // XOR with accumulator zeropage 3 cycles, 2 bytes
                        self.addressing_mode_zero_page_read(&Self::instruction_xor);
                    },
                    0x08 => {
                        // PHA Push Accumulator on Stack 3 cycles, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.ram.set_address(self.AC, self.SP as usize);
                                self.SP -= 1;
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x09 => {
                        // XOR with accumulator #immediate 2 cycles, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_xor);
                    },
                    0x0D => {
                        // XOR with accumulator absolute 4 cycles, 3 bytes
                        self.addressing_mode_absolute_read(&Self::instruction_xor);
                    },
                    _ => {}
                }
            },
            0x50 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BVC
                        self.addressing_mode_relative(&Self::instruction_bvc);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_xor);
                    },
                    0x05 => {
                        //XOR with accumulator zeropage X 4 cycles, 2 bytes
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_xor);
                    },
                    0x08 => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_cli);
                    },
                    0x09 => {
                        //XOR with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_xor);
                    },
                    0x0D => {
                        //XOR with accumulator absolute X 5 cycles, 3 bytes
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_xor);
                    },
                    _ => {}
                }
            },
            0x60 => {
                match nibble & 0x0F {
                    0x01 => {
                        // ADC indirect X 6 cycle, 2 bytes
                        self.addressing_mode_indirect_x_read(&Self::instruction_adc);
                    },
                    0x05 => {
                        // ADC 3 cycle, 2 bytes
                        self.addressing_mode_zero_page_read(&Self::instruction_adc);
                    },
                    0x08 => {
                        // PLA 4 cycles, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.cycle += 1;
                            },
                            0x2 => {
                                self.SP += 1;
                                self.cycle += 1;
                            },
                            0x3 => {
                                self.AC = self.ram.get_instruction(self.SP as usize);
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x09 => {
                        // ADC immediate 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_adc);
                    },
                    0x0D => {
                        // ADC absolute 4 cycle, 3 bytes
                        self.addressing_mode_absolute_read(&Self::instruction_adc);
                    },
                    _ => {}
                }
            },
            0x70 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BVS
                        self.addressing_mode_relative(&Self::instruction_bvs);
                    },
                    0x01 => {
                        // ADC indirect Y 6 cycle, 2 bytes
                        self.addressing_mode_indirect_y_read(&Self::instruction_adc);
                    },
                    0x05 => {
                        // ADC 4 cycle, 2 bytes
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_adc);
                    },
                    0x08 => {
                        //SEI 2 cycle, 1 byte
                        match self.cycle {
                            0x00 => {
                                self.new_instruction(nibble);
                                self.cycle = 1;
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
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_adc);
                    },
                    0x0D => {
                        // ADC absolute X 4* cycle, 3 bytes
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_adc);
                    },
                    _ => {}
                }
            },
            0x80 => {
                match nibble & 0x0F {
                    0x01 => {
                        // STA indirect X 6 cycles, 2 bytes
                        self.addressing_mode_indirect_x_write(&Self::instruction_sta);
                    },
                    0x04 => {
                        // STX zero, 4 cycle, 3 bytes
                        self.addressing_mode_zero_page_write(&Self::instruction_sty);
                    },
                    0x05 => {
                        // STA 3 cycle, 2 bytes
                        self.addressing_mode_zero_page_write(&Self::instruction_sta);
                    },
                    0x06 => {
                        // STX zero, 4 cycle, 3 bytes
                        self.addressing_mode_zero_page_write(&Self::instruction_stx);
                    },
                    0x08 => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_dey);
                    }
                    0x0A => {
                        // TXA 2 cycle, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.AC = self.X;
                                self.SR |= self.AC & 0x80;
                                self.SR |= if self.AC == 0x0 {0x2} else {0x0};
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x0C => {
                        // STX absolute, 4 cycle, 3 bytes
                        self.addressing_mode_absolute_write(&Self::instruction_sty);
                    },
                    0x0D => {
                        // STA absolute, 4 cycle, 3 bytes
                        self.addressing_mode_absolute_write(&Self::instruction_sta);
                    },
                    0x0E => {
                        // STX absolute, 4 cycle, 3 bytes
                        self.addressing_mode_absolute_write(&Self::instruction_stx);
                    },
                    _ => {}
                }
            },
            0x90 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BCC
                        self.addressing_mode_relative(&Self::instruction_bcc);
                    },
                    0x01 => {
                        // STA indirect Y 6 cycles, 2 bytes
                        self.addressing_mode_indirect_y_write(&Self::instruction_sta);
                    },
                    0x04 => {
                        // STY zero X, 4 cycle, 3 bytes
                        self.addressing_mode_zero_page_with_index_write(true, &Self::instruction_sty);
                    },
                    0x05 => {
                        // STA X 4 cycle, 2 bytes
                        self.addressing_mode_zero_page_with_index_write(true, &Self::instruction_sta);
                    },
                    0x06 => {
                        // STX zero page Y, 4 cycle, 3 bytes
                        self.addressing_mode_zero_page_with_index_write(false, &Self::instruction_stx);
                    },
                    0x09 => {
                        //STA absolute Y, 5 cycle
                        self.addressing_mode_absolute_with_index_write(false, &Self::instruction_sta);
                    },
                    0x0A => {
                        // TXS 2 cycle, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.SP = self.X;
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x0D => {
                        //STA absolute X, 5 cycle
                        self.addressing_mode_absolute_with_index_write(true, &Self::instruction_sta);
                    },
                    _ => {}
                }
            },
            0xA0 => {
                match nibble & 0x0F {
                    0x00 => {
                        self.addressing_mode_immediate(&Self::instruction_ldy);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_lda);
                    },
                    0x02 => {
                        // LDX 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_ldx);
                    },
                    0x04 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_ldy);
                    },
                    0x05 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_lda);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_ldx);
                    },
                    0x07 => {},
                    0x08 => {},
                    0x09 => {
                        // LDA #$x immediate, 2 cycle, 2 bytes
                        self.addressing_mode_immediate(&Self::instruction_lda);
                    },
                    0x0A => {
                        // TAX transfer accumulator to index x (1byte)(2 cycle)
                        match self.cycle {
                            0x00 => {
                                self.cycle = 1;
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
                    0x0C => {
                        self.addressing_mode_absolute_read(&Self::instruction_ldy);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_read(&Self::instruction_lda);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_read(&Self::instruction_ldx);
                    },
                    _ => {},
                }
            },
            0xB0 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BCS
                        self.addressing_mode_relative(&Self::instruction_bcs);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_lda);
                    },
                    0x04 => {
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_ldy);
                    }
                    0x05 => {
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_lda);
                    },
                    0x06 => {
                        self.addressing_mode_zero_page_with_index_read(false, &Self::instruction_ldx);
                    },
                    0x08 => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_clv);
                    },
                    0x09 => {
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_lda);
                    },
                    0x0C => {
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_ldy);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_lda);
                    },
                    0x0E => {
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_ldx);
                    },
                    _ => {}
                }
            },
            0xC0 => {
                match nibble & 0x0F {
                    0x00 => {
                        self.addressing_mode_immediate(&Self::instruction_cpy);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_cmp);
                    },
                    0x04 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_cpy);
                    },
                    0x05 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_cmp);
                    },
                    0x09 => {
                        self.addressing_mode_immediate(&Self::instruction_cmp);
                    },
                    0x0A => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_dex);
                    },
                    0x0C => {
                        self.addressing_mode_absolute_read(&Self::instruction_cpy);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_read(&Self::instruction_cmp);
                    },
                    _ => {}
                }
            },
            0xD0 => {
                match nibble & 0x0F {
                    0x00 => {
                        // BNE change branch if result was not zero
                        self.addressing_mode_relative(&Self::instruction_bne);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_cmp);
                    },
                    0x05 => {
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_cmp);
                    },
                    0x08 => {
                        self.addressing_mode_implied_or_accumulator(&Self::instruction_cld);
                    },
                    0x09 => {
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_cmp);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_cmp);
                    },
                    _ => {}
                }
            },
            0xE0 => {
                match nibble & 0x0F {
                    0x00 => {
                        self.addressing_mode_immediate(&Self::instruction_cpx);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_x_read(&Self::instruction_sbc);
                    },
                    0x04 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_cpx);
                    },
                    0x05 => {
                        self.addressing_mode_zero_page_read(&Self::instruction_sbc);
                    },
                    0x08 => {
                        // INX increment X (2cycle, 1 byte)
                        match self.cycle {
                            0x00 => {
                                self.cycle = 1;
                            },
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
                    0x09 => {
                        self.addressing_mode_immediate(&Self::instruction_sbc);
                    },
                    0x0A => {
                        // No operation NOP 2 cycles, 1 byte
                        match self.cycle {
                            0x0 => {
                                self.cycle = 1;
                            },
                            0x1 => {
                                self.reset_instruction();
                            },
                            _ => {}
                        }
                    },
                    0x0C => {
                        self.addressing_mode_absolute_read(&Self::instruction_cpx);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_read(&Self::instruction_sbc);
                    },
                    _ => {}
                }
            },
            0xF0 => {
                match nibble & 0x0F {
                    0x00 => {
                        self.addressing_mode_relative(&Self::instruction_beq);
                    },
                    0x01 => {
                        self.addressing_mode_indirect_y_read(&Self::instruction_sbc);
                    },
                    0x05 => {
                        self.addressing_mode_zero_page_with_index_read(true, &Self::instruction_sbc);
                    },
                    0x09 => {
                        self.addressing_mode_absolute_with_index_read(false, &Self::instruction_sbc);
                    },
                    0x0D => {
                        self.addressing_mode_absolute_with_index_read(true, &Self::instruction_sbc);
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        println!("after registers AC: {:#X?}, X: {:#X?}, Y: {:#X?}, SP: {:#X?}, PC: {:?}, SR: {:08b}", self.AC, self.X, self.Y, self.SP, self.PC, self.SR);
        println!();
    }

    fn new_instruction(&mut self, instruction: u8) {
        self.new_instruction = false;
        self.current_instruction = instruction;
    }

    fn reset_instruction(&mut self) {
        self.new_instruction = true;
        self.cycle = 0;
    }

    /**
     * instructions
     * function's name starts with `instruction_`
     */
    // and with accumulator
    fn instruction_and(&mut self, byte: u8) {
        self.AC &= self.ram.get_instruction(byte as usize);
        self.set_flag_1st_bit_zero(self.AC);
        self.set_flag_7th_bit_nagetive(self.AC);
    }

    // or with accumulator
    fn instruction_or(&mut self, byte: u8) {
        self.AC |= self.ram.get_instruction(byte as usize);
        self.set_flag_1st_bit_zero(self.AC);
        self.set_flag_7th_bit_nagetive(self.AC);
    }

    // xor with accumulator
    fn instruction_xor(&mut self, byte: u8) {
        self.AC ^= self.ram.get_instruction(byte as usize);
        self.set_flag_1st_bit_zero(self.AC);
        self.set_flag_7th_bit_nagetive(self.AC);
    }

    // add with carry
    fn instruction_adc(&mut self, byte: u8) {
        let sum:i16 = (self.AC as i16) + (byte as i16) + ((self.SR & 0x1) as i16);
        let sum_as_u8 = sum as u8;
        println!("sum: {:#b} {} {}", sum_as_u8, self.SR & 0x1, sum as i8);
        self.set_flag_0th_bit_carry(sum as u16);
        self.set_flag_1st_bit_zero(sum_as_u8);
        self.set_flag_6th_bit_overflow(self.AC as u16, byte as u16, sum as u16);
        self.set_flag_7th_bit_nagetive(sum_as_u8);
        self.AC = sum_as_u8;
    }

    // load into X
    fn instruction_ldx(&mut self, byte: u8) {
        self.X = byte;
        self.set_flag_1st_bit_zero(byte);
        self.set_flag_7th_bit_nagetive(byte);
    }

    // load into Y
    fn instruction_ldy(&mut self, byte: u8) {
        self.Y = byte;
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
    fn instruction_sta(&mut self, _: u8) -> u8 {
        //self.ram.set_address(self.AC, self.arg as usize);
        self.AC
    }

    // store X in memory {
    fn instruction_stx(&mut self, _: u8) -> u8 {
        //self.ram.set_address(self.X, self.arg as usize);
        self.X
    }

    // store Y in memory
    fn instruction_sty(&mut self, _: u8) -> u8 {
        //self.ram.set_address(self.Y, self.arg as usize);
        self.Y
    }

    // arithmetic shift left
    fn instruction_asl_accumulator(&mut self) {
        self.SR |= self.AC & 0x80;
        self.AC <<= 1;
        self.set_flag_7th_bit_nagetive(self.AC);
        self.set_flag_1st_bit_zero(self.AC);
    }

    fn instruction_asl_memory(&mut self, byte: u8) -> u8 {
        self.SR |= byte & 0x80;
        let byte = byte << 1;
        self.set_flag_7th_bit_nagetive(byte);
        self.set_flag_1st_bit_zero(byte);
        byte
    }

    // arithmetic shift left
    fn instruction_rol_accumulator(&mut self) {
        let carry = (self.SR & 0x80) >> 7;
        self.SR |= self.AC & 0x80;
        self.AC <<= 1;
        self.AC |= carry;
        self.set_flag_7th_bit_nagetive(self.AC);
        self.set_flag_1st_bit_zero(self.AC);
    }

    fn instruction_rol_memory(&mut self, byte: u8) -> u8 {
        let carry = (self.SR & 0x80) >> 7;
        self.SR |= byte & 0x80;
        let byte = (byte << 1) | carry;
        self.set_flag_7th_bit_nagetive(byte);
        self.set_flag_1st_bit_zero(byte);
        byte
    }

    // subtract with borrow
    fn instruction_sbc(&mut self, byte: u8) {
        let sum:i16 = (self.AC as i16) - (byte as i16) - ((self.SR & 0x1) as i16);
        let sum_as_u8 = sum as u8;
        println!("sum: {:#b} {} {}", sum_as_u8, self.SR & 0x1, sum as i8);
        self.set_flag_0th_bit_carry(sum as u16);
        self.set_flag_1st_bit_zero(sum_as_u8);
        self.set_flag_6th_bit_overflow(self.AC as u16, byte as u16, sum as u16);
        self.set_flag_7th_bit_nagetive(sum_as_u8);
        self.AC = sum_as_u8;
    }

    fn instruction_cmp(&mut self, byte: u8) {
        let diff:i16 = (self.AC as i16) - (byte as i16);
        let diff_as_u8 = diff as u8;
        if self.AC >= byte {
            self.SR |= 0x01;
        } else {
            self.SR &= 0xFE; //0xFE == 1111 1110 in binary
        }
        self.set_flag_1st_bit_zero(diff_as_u8);
        self.set_flag_7th_bit_nagetive(diff_as_u8);
    }

    fn instruction_cpx(&mut self, byte: u8) {
        let diff:i16 = (self.X as i16) - (byte as i16);
        let diff_as_u8 = diff as u8;
        println!("cpx:: byte {}", byte);
        if self.X >= byte {
            self.SR |= 0x01;
        } else {
            self.SR &= 0xFE; //0xFE == 1111 1110 in binary
        }
        self.set_flag_1st_bit_zero(diff_as_u8);
        self.set_flag_7th_bit_nagetive(diff_as_u8);
    }

    fn instruction_cpy(&mut self, byte: u8) {
        let diff:i16 = (self.Y as i16) - (byte as i16);
        let diff_as_u8 = diff as u8;
        if self.Y >= byte {
            self.SR |= 0x01;
        } else {
            self.SR &= 0xFE; //0xFE == 1111 1110 in binary
        }
        self.set_flag_1st_bit_zero(diff_as_u8);
        self.set_flag_7th_bit_nagetive(diff_as_u8);
    }

    fn instruction_bit(&mut self, byte: u8) {
        self.set_flag_7th_bit_nagetive(byte);
        if byte & 0x40 == 0x40 {
            self.SR |= 0x40;
        } else {
            self.SR &= 0xBF; //0xBF == 1011 1111 in binary
        }
        self.set_flag_1st_bit_zero(byte & self.AC);
    }

    fn instruction_dex(&mut self) {
        self.X -= 1;
        self.set_flag_1st_bit_zero(self.X);
        self.set_flag_7th_bit_nagetive(self.X);
    }

    fn instruction_dey(&mut self) {
        self.Y -= 1;
        self.set_flag_1st_bit_zero(self.X);
        self.set_flag_7th_bit_nagetive(self.X);
    }

    fn instruction_clc(&mut self) {
        self.SR &= 0xFE;
    }

    fn instruction_cld(&mut self) {
        self.SR &= 0xF7;
    }

    fn instruction_cli(&mut self) {
        self.SR &= 0xFB;
    }

    fn instruction_clv(&mut self) {
        self.SR &= 0xBF;
    }

    fn instruction_bpl(&mut self) -> bool {
        self.SR & 0x80 > 0
    }

    fn instruction_bmi(&mut self) -> bool {
        self.SR & 0x80 == 0
    }

    fn instruction_bne(&mut self) -> bool {
        self.SR & 0x02 > 0
    }

    fn instruction_bcc(&mut self) -> bool {
        self.SR & 0x01 > 0
    }

    fn instruction_bcs(&mut self) -> bool {
        self.SR & 0x01 == 0
    }

    fn instruction_bvc(&mut self) -> bool {
        self.SR & 0x40 > 0
    }

    fn instruction_bvs(&mut self) -> bool {
        self.SR & 0x40 == 0
    }

    fn instruction_beq(&mut self) -> bool {
        self.SR & 0x02 == 0
    }

    /**
     * set flags
     * function's name starts with `set_flag_`
     */
    fn set_flag_0th_bit_carry(&mut self, result:u16) {
        if result > 0xFF {
            self.SR |= 0x01;
        } else {
            self.SR &= 0xFE; //0xFE == 1111 1110 in binary
        }
    }

    fn set_flag_1st_bit_zero(&mut self, result:u8) {
        if result == 0x0 {
            self.SR |= 0x02;
        } else {
            self.SR &= 0xFD; // 0xFC == 1111 1101 in binary
        }
    }

    fn set_flag_6th_bit_overflow(&mut self, x:u16, y:u16, z:u16) {
        // The overflow flag is set when the sign of the addends is the same and
        // differs from the sign of the sum
        // overflow = <'x' and 'y' have the same sign> &
        //           <the sign of 'x' and 'sum' differs> &
        //           <extract sign bit>
        if (((z ^ x) & (z ^ y) & 0x80) as u8) > 0x0 {
            self.SR |= 0x40;
        } else {
            self.SR &= 0xBF;
        }
    }

    fn set_flag_7th_bit_nagetive(&mut self, result: u8) {
        if (result as i8) < 0 {
            self.SR |= 0x80;
        } else {
            self.SR &= 0x7F;
        }
        //self.SR |= result & 0x80;
    }

    /**
     * addressing modes
     * function's name starts with `addressing_mode_`
     */
    fn addressing_mode_immediate(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                let byte = self.ram.get_instruction(self.PC as usize);
                self.PC += 1;
                instruction(self, byte);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT, LAX, NOP)
    fn addressing_mode_zero_page_read(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                instruction(self, self.ram.get_instruction(self.arg as usize));
                self.reset_instruction();
            },
            _ => {}
        }
    }

    // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC, SLO, SRE, RLA, RRA, ISB, DCP)
    fn addressing_mode_zero_page_read_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                self.arg = self.ram.get_instruction(self.arg as usize) as u16;
                self.cycle += 1;
            },
            0x3 => {
                self.cycle += 1;
            },
            0x4 => {
                let byte = instruction(self, self.arg as u8);
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_zero_page_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
            },
            0x2 => {
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }


    // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT, LAX, NOP)
    fn addressing_mode_zero_page_with_index_read(&mut self, is_x: bool, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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

    fn addressing_mode_zero_page_with_index_read_write(&mut self, is_x: bool, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                self.cycle += 1;
            },
            0x4 => {
                self.cycle += 1;
            },
            0x5 => {
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_zero_page_with_index_write(&mut self, is_x: bool, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_absolute_read(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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

    fn addressing_mode_absolute_read_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                self.cycle += 1;
            },
            0x4 => {
                self.cycle += 1;
            },
            0x5 => {
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            }
            _ => {}
        }
    }

    fn addressing_mode_absolute_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_absolute_with_index_read(&mut self, is_x:bool, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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

    fn addressing_mode_absolute_with_index_read_write(&mut self, is_x:bool, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                self.cycle += 1;
            },
            0x5 => {
                self.cycle += 1;
            },
            0x6 => {
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }


    fn addressing_mode_absolute_with_index_write(&mut self, is_x:bool, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_indirect_x_read(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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


    fn addressing_mode_indirect_x_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_indirect_y_read(&mut self, instruction: &Fn(&mut Self, u8)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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

    fn addressing_mode_indirect_y_write(&mut self, instruction: &Fn(&mut Self, u8) -> u8) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
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
                let byte = instruction(self, self.ram.get_instruction(self.arg as usize));
                self.ram.set_address(byte, self.arg as usize);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_implied_or_accumulator(&mut self, instruction: &Fn(&mut Self)) {
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                instruction(self);
                self.reset_instruction();
            },
            _ => {}
        }
    }

    fn addressing_mode_relative(&mut self, instruction: &Fn(&mut Self) -> bool) {
        // BME change branch if result was negative
        match self.cycle {
            0x0 => {
                self.cycle = 1;
            },
            0x1 => {
                self.arg = self.ram.get_instruction(self.PC as usize) as u16;
                self.PC += 1;
                self.cycle += 1;
                if instruction(self) {
                    self.reset_instruction();
                }
            },
            0x2 => {
                println!("PC1: argI16: {}", self.arg as i8);
                let high = (self.PC >> 8) as u8;
                let low = (self.PC & 0xFF) as u8;
                println!("low : {}", low as i8);
                println!("high: {}", (high as u16));
                println!("signed {}", self.arg as i8 + low as i8);
                //println!("high+low {}",((high as u16) <<8) |(low+(self.arg as u8)) as u16);
                self.PC = (high as u16)<<8 | (low as i8 + self.arg as i8) as u16;

                println!("PC2: argI16: {}", self.arg as i16);

                // TODO:: fix PC high byte
                self.cycle += 1;
                self.reset_instruction();
            },
            0x3 => {
                self.reset_instruction();
            }
            _ => {}
        }
    }
}