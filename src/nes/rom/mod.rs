// An iNES file consists of the following sections, in order:
//
// Header (16 bytes)
// Trainer, if present (0 or 512 bytes)
// PRG ROM data (16384 * x bytes)
// CHR ROM data, if present (8192 * y bytes)
// PlayChoice INST-ROM, if present (0 or 8192 bytes)
// PlayChoice PROM, if present (16 bytes Data, 16 bytes CounterOut) (this is often missing, see PC10 ROM-Images for details)
// Some ROM-Images additionally contain a 128-byte (or sometimes 127-byte) title at the end of the file.


pub trait Rom {
    fn new(data: &Vec<u8>) -> Self;
    fn get_header(&self) -> &Header;
    fn get_rom_data(&self) -> &Vec<u8>;
    fn get_prg_data(&self) -> &Vec<u8>;
//    fn get_trainer(&self) -> Vec<u16>;
//    fn get_prg_rom_data(&self) -> Vec<u16>;
//    fn get_chr_rom_data(&self) -> Vec<u8>;
//    fn get_inst_rom(&self) -> Vec<u16>;
}


// The format of the header is as follows:
//
// 0-3: Constant $4E $45 $53 $1A ("NES" followed by MS-DOS end-of-file)
// 4: Size of PRG ROM in 16 KB units
// 5: Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
// 6: Flags 6 - Mapper, mirroring, battery, trainer
// 7: Flags 7 - Mapper, VS/Playchoice, NES 2.0
// 8: Flags 8 - PRG-RAM size (rarely used extension)
// 9: Flags 9 - TV system (rarely used extension)
// 10: Flags 10 - TV system, PRG-RAM presence (unofficial, rarely used extension)
// 11-15: Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15)
pub struct Header {
    first_four_constants : [u8;4],
    prg_rom_size: u8,
    chr_rom_size: u8,

    //flags 6 to 10
    flags: [u8; 5],
}

impl Header {

    pub fn new(data:&[u8]) -> Header {
        Header {
            first_four_constants:[data[0], data[1], data[2], data[3]],
            prg_rom_size: data[4],
            chr_rom_size: data[5],
            flags: [data[6], data[7], data[8], data[9], data[10]]
        }
    }

    pub fn get_constants(&self) -> [u8;4]{
        self.first_four_constants
    }
    pub fn constant_as_str(&self) -> String {
        self.first_four_constants.iter().map(|x|*x as char).collect()
    }
    pub fn get_prg_rom_size(&self) -> u8 {
        self.prg_rom_size
    }
    pub fn get_chr_rom_size(&self) -> u8 {
        self.chr_rom_size
    }
    pub fn get_all_flag(&self) -> [u8;5] {
        self.flags
    }
    pub fn get_flag(&self, idx: usize) -> u8 {
        self.flags[idx]
    }
}

pub struct RomV1 {
    header: Header,
    prg: Vec<u8>
}


impl Rom for RomV1 {
    fn new(data: &Vec<u8>) -> RomV1 {
        let header = Header::new(&data[0..11]);
        let to = (0x4000*(header.prg_rom_size as u32) + 0xF) as usize;
        RomV1 {
            header: header,
            prg: data[0x10..to].to_vec()
        }
    }

    fn get_header(&self) -> &Header {
        &self.header
    }

    fn get_prg_data(&self) -> &Vec<u8> { &self.prg }

    fn get_rom_data(&self) -> &Vec<u8> {
        &self.prg
    }
}
