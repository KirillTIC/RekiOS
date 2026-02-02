<div align="center">
  <img width="120" height="120" alt="RekiOS logo" src="https://github.com/user-attachments/assets/1af031f4-d30a-4744-b9b7-f6b5d2472673"/>
  
  # Reki OS
  
  **A lightweight operating system written entirely in Rust**
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
  
</div>

---

## 🏛️ About

Reki OS is a minimalist operating system project built from the ground up using Rust. It focuses on safety, performance, and simplicity.

## ✨ Features

- **Memory Safe**: Leverages Rust's ownership system for safe memory management
- **Lightweight**: Minimal footprint and fast boot times
- **Modern**: Built with cutting-edge systems programming practices

## 🛠️ Building
```bash
# Clone the repository
git clone https://github.com/KirillTIC/RekiOS.git
cd RekiOS

# Build the OS
./build.sh

# Run in QEMU 
qemu-system-x86_64 -drive format=raw,file=target/x86_64-reki_os/debug/bootimage-reki_os.bin
```

## 📋 Requirements to build

- Rust 1.70 or higher
- bootimage cargo

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built with ❤️ using only Rust
