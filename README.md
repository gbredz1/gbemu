# Gameboy Emulator with Debugger
Written in Rust, using *iced* for main GUI and terminal version with *Ratatui*.
This project serves as a personal learning exercise to learn and gain experience with both libraries.

Download the latest Nightly builds from [Releases](https://github.com/gbredz1/gbemu/releases)

[![Linux](https://img.shields.io/badge/Linux-x86_64-orange)](https://github.com/gbredz1/gbemu/releases)  
[![macOS ARM](https://img.shields.io/badge/macOS-ARM-blue)](https://github.com/gbredz1/gbemu/releases)  
[![macOS x86_64](https://img.shields.io/badge/macOS-x86_64-blue)](https://github.com/gbredz1/gbemu/releases)  
[![Windows](https://img.shields.io/badge/Windows-x86_64_MSVC-red)](https://github.com/gbredz1/gbemu/releases)

---

### :construction: WORK IN PROGRESS :construction:
:warning: The emulator currently runs only some games, and graphical glitches are present

- MBC1 mapper implementation passes the [Mooneye test suite](https://github.com/Gekkio/mooneye-test-suite) ✅

---

### Build and Run
#### Iced Frontend

```bash
cargo run --release --bin gbemu-iced
```

![desktop iced screenshot](https://i.ibb.co/r2Kt5RFC/screenshot-001.png)

#### Terminal (experiment)

```bash
cargo run --release --bin gbemu-term -- roms/test.gb
```

![terminal screenshot](https://i.ibb.co/bR1SBNjz/screenshot-002.png)
