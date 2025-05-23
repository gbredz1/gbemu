# Gameboy Emulator with Debugger
Written in Rust, using *iced* for main GUI and terminal version with *Ratatui*.
This project serves as a personal learning exercise to learn and gain experience with both libraries.

### :construction: WORK IN PROGRESS :construction:
:warning: Emulator not running at this stage.

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
