mod nes;

use crate::nes::rom::Rom;

use std::io;

// For testing
#[cfg(feature="debug")]
fn start() {
    let path = "resources/test";

    let roms = nes::loader::list_test_roms(path);
    let roms_with_idx = (1..roms.len()).zip(roms).collect::<Vec<_>>();
    loop {
        roms_with_idx.iter().for_each(|(ind, path)|{
            println!("{} {:?}", ind, path);

        });
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input).and_then(|_x|
            match input.trim().parse::<usize>() {
                Ok(line) => {
                    println!("loading game {:?} {:?}", line, &roms_with_idx[line-1].1.path().to_str().unwrap());
                    let rom_data = nes::loader::load_rom(&roms_with_idx[line-1].1.path().to_str().unwrap());
                    let rom: nes::rom::RomV1 = nes::rom::Rom::new(&rom_data.unwrap());
                    let mut processor = nes::cpu::processor::Processor::new(rom.get_rom_data());
                    let mut i = 0;
                    loop {
                        processor.execute_next_instruction();
                        i += 1;
                        // only for testing
                        // TODO :: remove this break
                        if i > 60 {
                            break;
                        }
                    }
                    //nes::loader::load_rom(&roms_with_idx[line-1].1.path().to_str().unwrap()).unwrap().iter().for_each(|x|
                    //    print!("{:#04X?}, ",x)
                    //);
                    println!();
                    println!("header constant:: {:?}", rom.get_header().constant_as_str());
                    println!("header prg size:: {:#x?}", rom.get_header().get_prg_rom_size());
                    println!("header chr size:: {:#x?}", rom.get_header().get_chr_rom_size());
                    println!("header flags:: {:#x?}", rom.get_header().get_all_flag());
                    Ok(())
                },
                Err(_e) => {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid Input"))
                }
            }
        );
        break;
    }

}

#[cfg(feature="release")]
fn start() {
    //TODO:: logic for release
    println!("Main logic");
}

fn main() {
    println!("Hello, world!");
    start();
}
