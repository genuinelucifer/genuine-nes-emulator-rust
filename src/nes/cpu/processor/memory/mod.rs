//
//0000-07FF is RAM, 0800-1FFF are mirrors of RAM (you AND the address with 07FF to get the effective address)
//2000-2007 is how the CPU writes to the PPU, 2008-3FFF are mirrors of that address range.
//4000-401F is for IO ports and sound
//4020-4FFF is rarely used, but can be used by some cartridges
//5000-5FFF is rarely used, but can be used by some cartridges, often as bank switching registers, not actual memory, but some cartridges put RAM there
//6000-7FFF is often cartridge WRAM. Since emulators usually emulate this whether it actually exists in the cartridge or not, there's a little bit of controversy about NES headers not adequately representing a cartridge.
//8000-FFFF is the main area the cartridge ROM is mapped to in memory. Sometimes it can be bank switched, usually in 32k, 16k, or 8k sized banks.

pub struct Memory {
    ram: [u8; 0x800], // 2KB Ram
    ppu: [u8; 0x8],
    apu: [u8; 0x2000],
    wram: [u8; 0x2000],
    prg: [u8; 0x8000]
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            ram: [0; 0x800],
            ppu: [0; 0x8],
            apu: [0; 0x2000],
            wram: [0; 0x2000],
            prg:[0; 0x8000]
        }
    }

    pub fn load(&mut self, prg: &Vec<u8>, from: usize) {
        let mut pos = from;
        for byte in prg.iter() {
            self.prg[pos - 0x8000] = *byte;
            pos += 1;
        }
    }

    pub fn set_address(&mut self, data: u8, at: usize) {
        if at < 0x2000 {  // 2KB Ram
            self.ram[at % 0x8FF] = data;
        } else if at < 0x4000 { // PPU
            self.ppu[(at - 0x2000) % 0x8] = data;
        } else if at < 0x6000 { // APU
            self.wram[at - 0x4000] = data;
        } else if at < 0x8000 { // WRAM
            self.wram[at - 0x6000] = data;
        } else { // PRG ROM
            self.prg[at - 0x8000] = data;
        }
    }

    pub fn get_instruction(&self, at: usize) -> u8 {
        if at < 0x2000 {  // 2KB Ram
            self.ram[at % 0x8FF]
        } else if at < 0x4000 { // PPU
            self.ppu[(at - 0x2000) % 0x8]
        } else if at < 0x6000 { // APU
            self.wram[at - 0x4000]
        } else if at < 0x8000 { // WRAM
            self.wram[at - 0x6000]
        } else { // PRG ROM
            self.prg[at - 0x8000]
        }
    }
}