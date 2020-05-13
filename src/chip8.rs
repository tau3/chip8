use std::fs::File;
use std::io::Read;

use rand::Rng;

use crate::buffer::Buffer;

pub struct Chip8 {
    opcode: u16,
    memory: Buffer<u8>,
    vr: Buffer<u8>,
    ir: u16,
    pc: u16,
    gfx: Buffer<u8>,
    delay_timer: u8,
    sound_timer: u8,
    stack: Buffer<u16>,
    sp: u16,
    key: Buffer<u8>,
    draw_flag: bool,
}

impl Chip8 {
    // TODO refactor
    pub fn new() -> Self {
        // reset everything
        let opcode: u16 = 0;
        let mut memory = Buffer::new(4096);
        let vr = Buffer::new(16);
        let ir: u16 = 0;
        let pc: u16 = 0x200;
        let gfx = Buffer::new(WIDTH * HEIGHT);
        let delay_timer: u8 = 0;
        let sound_timer: u8 = 0;
        let stack = Buffer::new(16);
        let sp: u16 = 0;
        let key = Buffer::new(16);
        let draw_flag = false;

        // load fontset
        for i in 0..80 {
            memory[i] = CHIP8_FONTSET[i];
        }

        Chip8 {
            opcode,
            memory,
            vr,
            ir,
            pc,
            gfx,
            delay_timer,
            sound_timer,
            stack,
            sp,
            key,
            draw_flag,
        }
    }

    pub fn load_game(&mut self, name: &str) {
        let mut file = File::open(name).expect("failed to open game");
        match file.read_exact(self.memory.slice_from(512)) {
            Err(ref e) => {
                if e.kind() == std::io::ErrorKind::Interrupted { panic!("failed to read game") }
            }
            _ => {}
        };
    }

    pub fn emulate_cycle(&mut self) {
        // fetch opcode (two bytes)
        self.opcode = ((self.memory[self.pc] as u16) << 8) | self.memory[self.pc + 1] as u16;

        // decode opcode
        match self.opcode & FIRST_FOUR_BITS {
            0xA000 => {
                self.ir = self.opcode & LAST_TWELVE_BITS;
                self.pc += 2;
            }
            0xC000 => {
                let x = self.opcode.x();
                let nn = (self.opcode & LAST_EIGHT_BITS) as u8;
                let random_number: u8 = rand::thread_rng().gen_range(0, 255);
                self.vr[x] = nn & random_number;
                self.pc += 2;
            }
            0x8000 => {
                match self.opcode & LAST_FOUR_BITS {
                    0x0002 => {
                        let x = self.opcode.x();
                        let y = self.opcode.y();
                        self.vr[x] &= self.vr[y];
                        self.pc += 2;
                    }
                    0x0004 => {
                        let x = self.opcode.x();
                        if self.vr[(self.opcode & THIRD_FOUR_BITS) >> 4] > 0xFF - self.vr[x] {
                            self.vr[0xFu8] = 1u8; // carry
                        } else {
                            self.vr[0xFu8] = 0u8;
                        }
                        self.vr[x] = self.vr[x].wrapping_add(self.vr[(self.opcode & THIRD_FOUR_BITS) >> 4]);
                        self.pc += 2;
                    }
                    _ => { println!("[0x8000]: {:X} is not recognized", self.opcode) }
                }
            }
            0x0000 => {
                match self.opcode & LAST_FOUR_BITS {
                    0x000E => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp];
                        self.pc += 2;
                    }
                    _ => { println!("[0x0000]: {:X} is not recognized", self.opcode) }
                }
            }
            0x2000 => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & LAST_TWELVE_BITS;
            }
            0xD000 => {
                let x: u8 = self.vr[self.opcode.x()];
                let y: u8 = self.vr[(self.opcode & THIRD_FOUR_BITS) >> 4];
                let height: u8 = (self.opcode & LAST_FOUR_BITS) as u8;
                self.vr[0xFu8] = 0u8;
                for yline in 0..height {
                    let pixel = self.memory[self.ir + yline as u16];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let pos = x as usize + xline as usize + (y + yline) as usize * WIDTH;
                            if self.gfx[pos] == 1 {
                                self.vr[0xFu8] = 1u8;
                            }
                            self.gfx[pos] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => {
                match self.opcode & LAST_EIGHT_BITS {
                    0x009E => {
                        if self.key[self.vr[self.opcode.x()]] != 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        let x = self.opcode.x();
                        let vx = self.vr[x];
                        if self.key[vx] == 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => { println!("[0XE000]: {:X} is not recognized", self.opcode) }
                }
            }
            0x6000 => {
                // upper bits will be truncated while casting u16 to u8
                self.vr[self.opcode.x()] = (self.opcode & FIRST_EIGHT_BITS) as u8;
                self.pc += 2;
            }
            0x7000 => {
                let x = self.opcode.x();
                let nn = (self.opcode & FIRST_EIGHT_BITS) as u8;
                self.vr[x] += nn;
                self.pc += 2;
            }
            0x3000 => {
                let x = self.opcode.x();
                let nn = (self.opcode & FIRST_EIGHT_BITS) as u8;
                if self.vr[x] == nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x1000 => {
                let nnn = self.opcode & LAST_TWELVE_BITS;
                self.pc = nnn;
            }
            0xF000 => {
                match self.opcode & LAST_EIGHT_BITS {
                    0x0033 => {
                        self.memory[self.ir] = self.vr[self.opcode.x()] / 100;
                        self.memory[self.ir + 1] = (self.vr[self.opcode.x()] / 10) % 10;
                        self.memory[self.ir + 2] = (self.vr[self.opcode.x()] % 100) % 10;
                        self.pc += 2;
                    }
                    0x0065 => {
                        let x = self.opcode.x();
                        let mut offset = self.ir;
                        for i in 0..=x {
                            self.vr[i] = self.memory[offset];
                            offset += 1;
                        }
                        self.pc += 2;
                    }
                    0x0029 => {
                        let x = self.opcode.x();
                        self.ir = self.vr[x] as u16 * 5;
                        self.pc += 2;
                    }
                    0x0015 => {
                        let x = self.opcode.x();
                        self.delay_timer = x as u8;
                        self.pc += 2;
                    }
                    0x0007 => {
                        let x = self.opcode.x();
                        self.vr[x] = self.delay_timer;
                        self.pc += 2;
                    }
                    _ => { println!("[0xF000]: {:X} is not recognized", self.opcode) }
                }
            }
            _ => { println!("{:X} is not recognized", self.opcode) }
        }

        // update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                beep()
            }
            self.sound_timer -= 1;
        }
    }

    pub fn is_draw_flag(&self) -> bool {
        self.draw_flag
    }

    pub fn set_keys(&self) {}

    pub fn is_set(&self, row: usize, col: usize) -> bool {
        self.gfx[row * WIDTH + col] == 1
    }
}

fn beep() {}

static CHIP8_FONTSET: [u8; 80] = [
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

static FIRST_FOUR_BITS: u16 = 0xF000;
static LAST_TWELVE_BITS: u16 = 0x0FFF;
static LAST_FOUR_BITS: u16 = 0x000F;
static THIRD_FOUR_BITS: u16 = 0x00F0;
static SECOND_FOUR_BITS: u16 = 0x0F00;
static LAST_EIGHT_BITS: u16 = 0x00FF;
static FIRST_EIGHT_BITS: u16 = 0xFF;

pub static WIDTH: usize = 64;
pub static HEIGHT: usize = 32;

trait OpCode {
    fn x(&self) -> u16;

    fn y(&self) -> u16;
}

impl OpCode for u16 {
    fn x(&self) -> u16 {
        (self & SECOND_FOUR_BITS) >> 8
    }

    fn y(&self) -> u16 {
        (self & THIRD_FOUR_BITS) >> 4
    }
}
