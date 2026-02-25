use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let kernel = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&output)
        .unwrap();

    println!("Created UEFI image: {}", output.display());
}
