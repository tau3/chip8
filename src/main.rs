use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

fn main() {
    game_loop();
}

fn setup_graphics() {}

fn setup_input() {}

fn draw_graphics() {}

fn game_loop() {
    setup_graphics();
    setup_input();

    let mut chip8 = Chip8::new();
    chip8.load_game("pong2.c8");

    loop {
        thread::sleep(Duration::from_millis(1000 / 60));

        chip8.emulate_cycle();
        if chip8.is_draw_flag() {
            draw_graphics();
        }
        chip8.set_keys();
    }
}

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

static FIRST_FOUR_BITS_MASK_16: u16 = 0xF000;
static LAST_TWELVE_BITS_MASK_16: u16 = 0x0FFF;
static FOURTH_FOUR_BITS_MASK_16: u16 = 0x000F;
static THIRD_FOUR_BITS_MASK_16: u16 = 0x00F0;
static SECOND_FOUR_BITS_MASK_16: u16 = 0x0F00;

struct Buffer<T: Clone + From<u8>> {
    buffer: Vec<T>,
}

impl<T: Clone + From<u8>> Buffer<T> {
    fn new(size: usize) -> Buffer<T> {
        let memory = vec![(0 as u8).into(); size];
        Buffer { buffer: memory }
    }
}

impl<T: Clone + From<u8>> std::ops::Index<usize> for Buffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl<T: Clone + From<u8>> std::ops::Index<u16> for Buffer<T> {
    type Output = T;

    fn index(&self, index: u16) -> &Self::Output {
        &self.buffer[index as usize]
    }
}

impl<T: Clone + From<u8>> std::ops::Index<u8> for Buffer<T> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        &self.buffer[index as usize]
    }
}

impl<T: Clone + From<u8>> std::ops::Index<i32> for Buffer<T> {
    type Output = T;

    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[index as usize]
    }
}

struct Chip8 {
    opcode: u16,
    memory: Buffer<u8>,
    v_registers: Buffer<u8>,
    index_register: u16,
    program_counter: u16,
    gfx: Buffer<u8>,
    delay_timer: u8,
    sound_timer: u8,
    stack: Buffer<u16>,
    sp: u16,
    key: [u8; 16],
    draw_flag: bool,
}

impl Chip8 {
    fn new() -> Self {
        // reset everything
        let opcode: u16 = 5;
        let mut memory = Buffer::new(4096);
        let v_registers = Buffer::new(16);
        let index_register: u16 = 0;
        let program_counter: u16 = 0x200;
        let gfx = Buffer::new(64 * 32);
        let delay_timer: u8 = 0;
        let sound_timer: u8 = 0;
        let stack = Buffer::new(16);
        let sp: u16 = 0;
        let key: [u8; 16] = [0; 16];
        let draw_flag = false;

        // load fontset
        for i in 0..80 {
            memory[i] = CHIP8_FONTSET[i];
        }

        Chip8 {
            opcode,
            memory,
            v_registers,
            index_register,
            program_counter,
            gfx,
            delay_timer,
            sound_timer,
            stack,
            sp,
            key,
            draw_flag,
        }
    }

    fn load_game(&mut self, name: &str) {
        let mut file = File::open("pong").expect("failed to read game");
        file.read_exact(&mut self.memory[512..]);
    }

    fn emulate_cycle(&mut self) {
        // fetch opcode (two bytes)
        self.opcode = ((self.memory[self.program_counter] as u16) << 8) | self.memory[self.program_counter + 1].into();

        // decode opcode
        match self.opcode & FIRST_FOUR_BITS_MASK_16 {
            // ANNN: Sets index register to the address NNN
            0xA000 => {
                self.index_register = self.opcode & LAST_TWELVE_BITS_MASK_16;
                self.program_counter += 2;
            }
            0x0000 => {
                match self.opcode & FOURTH_FOUR_BITS_MASK_16 {
                    0x0000 => { /* 0x00E0: clear screen */ }
                    0x000E => { /* 0x000E: returns from subroutine */ }
                    _ => { println!("[0x0000]: {:X} is not recognized", self.opcode) }
                }
            }
            0x2000 => {
                self.stack[self.sp] = self.program_counter;
                self.sp += 1;
                self.program_counter = self.opcode & LAST_TWELVE_BITS_MASK_16;
            }
            0x0004 => {
                if self.v_registers[(self.opcode & THIRD_FOUR_BITS_MASK_16) >> 4] > 0xFF - self.v_registers[(self.opcode & 0xF00) >> 8] {
                    self.v_registers[0xF] = 1u8; // carry
                } else {
                    self.v_registers[0xF] = 0u8;
                }
                self.v_registers[(self.opcode & THIRD_FOUR_BITS_MASK_16 >> 8)] += self.v_registers[(self.opcode & THIRD_FOUR_BITS_MASK_16) >> 4];
                self.program_counter += 2;
            }
            0x0033 => {
                self.memory[self.index_register] = self.v_registers[(self.opcode & SECOND_FOUR_BITS_MASK_16) >> 8] / 100;
                self.memory[self.index_register + 1] = (self.v_registers[(self.opcode & SECOND_FOUR_BITS_MASK_16) >> 8] / 10) % 10;
                self.memory[self.index_register + 2] = (self.v_registers[(self.opcode & SECOND_FOUR_BITS_MASK_16) >> 8] % 100) % 10;
            }
            0xD000 => {
                let x: u8 = self.v_registers[(self.opcode & SECOND_FOUR_BITS_MASK_16) >> 8];
                let y: u8 = self.v_registers[(self.opcode & SECOND_FOUR_BITS_MASK_16) >> 4];
                let height: u8 = (self.opcode & FOURTH_FOUR_BITS_MASK_16) as u8;
                self.v_registers[0xF] = 0u8;
                for yline in 0..height {
                    let pixel = self.memory[self.index_register + yline as u16];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            if self.gfx[(x + xline + ((y + yline) * 64))] == 1 {
                                self.v_registers[0xf] = 1u8;
                            }
                            self.gfx[(x + xline + ((y + yline) * 64))] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
                self.program_counter += 2;
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

    fn is_draw_flag(&self) -> bool {
        true
    }

    fn set_keys(&self) {}
}

fn beep() {}