extern crate walkdir;

use walkdir::WalkDir;
use std::{fs, io};

// Only for `.nes` file
pub fn list_test_roms(path: &str) -> Vec<walkdir::DirEntry>{
    let mut roms = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".nes"))
        .collect::<Vec<walkdir::DirEntry>>();

    roms.sort_by(|x, y| String::from(x.path().to_string_lossy()).cmp(&String::from(y.path().to_string_lossy())));
    roms
}

pub fn load_rom(path: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(String::from(path).clone())
}