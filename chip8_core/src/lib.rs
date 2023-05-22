use rand::prelude::*;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    /// Program counter
    pc: u16,

    /// Ram
    ram: [u8; RAM_SIZE],

    /// Monocrome display
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],

    /// V-Registers
    v_reg: [u8; NUM_REGISTERS],

    /// I-Register
    i_reg: u16,

    /// Stack pointer - points at the top of the stack
    sp: u16,

    /// Stack - for storing subroutines address
    stack: [u16; STACK_SIZE],

    /// Keys - store the current key state
    keys: [bool; NUM_KEYS],

    /// Dealy timer - does some action with it hits 0
    dt: u8,

    /// Sound timre - emits a noise when it hits 0
    st: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGISTERS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        assert!(idx < 16, "Index cannot be greater than 15");
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = START_ADDR as usize + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGISTERS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();

        // Decode & Encode
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // beep
            }

            self.st -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        self.pc += 2;

        (higher_byte << 8) | lower_byte
    }

    fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        match (d1, d2, d3, d4) {
            // NOP
            (0, 0, 0, 0) => {  },

            // CLS
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // RET
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),

            // JMP NNN
            (1, _, _, _) => self.pc = op & 0xFFF,

            // CALL NNN
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0xFFF;
            }

            // SKIP VX == NN
            (3, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // SKIP VX != NN
            (4, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x] != nn as _ {
                    self.pc += 2;
                }
            }

            // SKIP VX == VY
            (5, _, _, _) => {
                let x = d2 as usize;
                let y = d3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // VX = NN
            (6, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = nn;
            }

            // VX += NN
            (7, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // VX = VY
            (8, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;

                self.v_reg[x] = self.v_reg[y];
            }

            // VX |= VY
            (8, _, _, 1) => {
                let x = d2 as usize;
                let y = d2 as usize;

                self.v_reg[x] |= self.v_reg[y];
            }

            // VX &= VY
            (8, _, _, 2) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },

            // VX ^= VY
            (8, _, _, 3) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },

            // VX += VY
            (8, _, _, 4) => {
                let x = d2 as usize;
                let y = d3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // VX -= VY
            (8, _, _, 5) => {
                let x = d2 as usize;
                let y = d3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // VX >> 1
            (8, _, _, 6) => {
                let x = d2 as usize;

                let drop = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = drop;
            }

            // VX = VY - VX
            (8, _, _, 7) => {
                let x = d2 as usize;
                let y = d3 as usize;

                let (new_vx, overflow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = overflow as _;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX << 1
            (8, _, _, 0xE) => {
                let x = d2 as usize;
                let drop = (self.v_reg[x] >> 7) & 1;

                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = drop;
            }

            // SKIP VX != VY
            (9, _, _, 0) => {
                let x = d2 as usize;
                let y = d2 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                let rand: u8 = rand::thread_rng().gen();
                self.v_reg[x] = rand & nn;
            }

            // DRAW DXYN - VX, VY
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[d2 as usize] as u16;
                let y_coord = self.v_reg[d3 as usize] as u16;
                let num_rows = d4;

                let mut flipped = false;
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },

            // SKIP KEY DOWN - key = VX
            (0xE, _, 9, 0xE) => {
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // SKIP KEY UP
            (0xE, _, 0xA, 1) => {
                let x = d2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            // VX = DT
            (0xF, _, 0, 7) => {
                let x = d2 as usize;
                self.v_reg[x] = self.dt;
            }

            // WAIT KEY
            (0xF, _, 0, 0xA) => {
                let x = d2 as usize;
                let mut pressed = false;

                for (i, key) in self.keys.iter().enumerate() {
                    if *key {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                // if not pressed redo this opcode
                if !pressed {
                    self.pc -= 2;
                }
            }

            // DT = VX
            (0xF, _, 1, 5) => {
                let x = d2 as usize;
                self.dt = self.v_reg[x];
            }

            // ST = VX
            (0xF, _, 1, 8) => {
                let x = d2 as usize;
                self.st = self.v_reg[x];
            }

            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = d2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            }

            // I = FONT
            (0xF, _, 2, 9) => {
                let x = d2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }

            // BCD
            (0xF, _, 3, 3) => {
                let x = d2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[self.i_reg as usize + 1] = tens;
                self.ram[self.i_reg as usize + 2] = ones;
            }

            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {op}"),
        }
    }
}
