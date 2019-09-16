//
//0000-07FF is RAM, 0800-1FFF are mirrors of RAM (you AND the address with 07FF to get the effective address)
//2000-2007 is how the CPU writes to the PPU, 2008-3FFF are mirrors of that address range.
//4000-401F is for IO ports and sound
//4020-4FFF is rarely used, but can be used by some cartridges
//5000-5FFF is rarely used, but can be used by some cartridges, often as bank switching registers, not actual memory, but some cartridges put RAM there
//6000-7FFF is often cartridge WRAM. Since emulators usually emulate this whether it actually exists in the cartridge or not, there's a little bit of controversy about NES headers not adequately representing a cartridge.
//8000-FFFF is the main area the cartridge ROM is mapped to in memory. Sometimes it can be bank switched, usually in 32k, 16k, or 8k sized banks.

pub struct Memory {
    data: [u8; 0x10000]
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data:[0; 0x10000]
        }
    }

    pub fn load(&mut self, data: &Vec<u8>, from: usize) {
        let mut pos = from;
        for byte in data.iter() {
            self.data[pos] = *byte;
            pos += 1;
        }
    }

    pub fn get_instruction(&self, idx: usize) -> u8 {
        self.data[idx]
    }
}