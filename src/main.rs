mod nes_loader;

// For testing
#[cfg(feature="debug")]
fn start() {
    let path = "resources/test";
    let roms = nes_loader::list_test_roms(path);
    roms.into_iter().for_each(|x|println!("{:?}", x));
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
