// Implementation of 2A03 chip used in NES CPU
// It consists of MOS 6502 processor and APU
// Apart from 6502, it also contains 22 extra registers for sound generation, joystick reading and OAM DMA transferring.

pub mod processor;

struct CPU {
    special_registers: [u8;22],
}