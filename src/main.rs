use std::fs::File;
use std::io::Read;
use std::panic::resume_unwind;

fn main() {
    let opcode: u16 = 5;
    let memory: [u8; 4096] = [0; 4096];
    let v_registers: [u8; 16] = [0; 16];
    let index_register: u16 = 0;
    let program_counter: u16 = 0;
    let gfx: [u8; 64 * 32] = [0; 64 * 32];
    let delay_timer: u8 = 0;
    let sound_timer: u8 = 0;

    let stack: [u16; 16] = [0; 16];
    let sp: u16;

    let key: [u8; 16] = [0; 16];
}

fn setup_graphics() {}

fn setup_input() {}

fn draw_graphics() {}

fn game_loop() {
    setup_graphics();
    setup_input();

    let chip8 = Chip8::new();
    chip8.initialize();
    chip8.load_game("pong");

    loop {
        // TODO 60 cycles per second
        chip8.emulate_cycle();
        if chip8.is_draw_flag() {
            draw_graphics();
        }
        chip8.set_keys();
    }
}

struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    v_registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    key: [u8; 16],
}

impl Chip8 {
    fn new() -> Self {
        // reset everything
        let opcode: u16 = 5;
        let mut memory: [u8; 4096] = [0; 4096];
        let v_registers: [u8; 16] = [0; 16];
        let index_register: u16 = 0;
        let program_counter: u16 = 0x200;
        let gfx: [u8; 64 * 32] = [0; 64 * 32];
        let delay_timer: u8 = 0;
        let sound_timer: u8 = 0;
        let stack: [u16; 16] = [0; 16];
        let sp: u16 = 0;
        let key: [u8; 16] = [0; 16];

        // load fontset
        for i in 0..80 {
            memory[i] = chip_fontset[i];
        }
        // load program
        read_binary(&mut memory[512..]);

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
        }
    }

    fn initialize(&self) {}
    fn load_game(&self, name: &str) {}
    fn emulate_cycle(&mut self) {
        // fetch opcode (two bytes)
        self.opcode = self.memory[self.program_counter << 8] | self.memory[self.program_counter + 1];

        // decode opcode (first four bits)
        match self.opcode & 0xF000 {
            0xA000 => {
                self.index_register = self.opcode & 0x0FFF; // last 12 bits
                self.pc += 2;
            }
            _ => { println!("{} is not recognized", self.opcode) }
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

fn read_binary(buffer: &mut [u8]) {
    let mut file = File::open("pong").expect("failed to read game");
    file.read_exact(buffer);
}

fn beep() {}