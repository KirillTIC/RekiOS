use std::path::PathBuf;

fn main() {
    let kernel = PathBuf::from(env!("CARGO_BIN_FILE_REKI_OS_reki_os"));
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let uefi_path = out_dir.join("uefi.img");

    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
}
