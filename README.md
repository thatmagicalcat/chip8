# Chip-8 Emulator
A chip8 emulator written in rust

## Project structure

```
chip8
├── chip8_core          - Backend
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── desktop             - Fontend
│   ├── Cargo.lock
│   ├── Cargo.toml
│   └── src
│       ├── main.rs
│       └── vars.rs
├── README.md
└── roms
    ├── ...
    └── ...
6 directories, 30 files

```

## Installation

``` shell
$ git clone https://github.com/Pranjal-Patel/chip8
$ cd chip8
$ cargo install --path desktop
```

## Usage

``` shell
$ chip8 /path/to/game
```

## Since this is my first project, I learned from this book
https://github.com/aquova/chip8-book
