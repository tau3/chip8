use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::chip8;
use crate::chip8::Chip8;

static SCALE: u32 = 10;

static BLACK: Color = Color::RGB(0, 0, 0);
static WHITE: Color = Color::RGB(255, 255, 255);

pub struct Presenter {
    chip8: Chip8,
}

impl Presenter {
    pub fn new(chip8: Chip8) -> Self {
        Presenter { chip8 }
    }

    // TODO refactor
    pub fn event_loop(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "chip8",
                SCALE * chip8::WIDTH as u32,
                SCALE * chip8::HEIGHT as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            if let Some(event) = event_pump.poll_event() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'running;
                    }
                    Event::KeyDown {
                        keycode: Some(keycode), ..
                    } => {
                        if let Some(chip_key_code) = key_to_code(keycode) {
                            self.chip8.set_keys(chip_key_code, true)
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode), ..
                    } => {
                        if let Some(chip_key_code) = key_to_code(keycode) {
                            self.chip8.set_keys(chip_key_code, false)
                        }
                    }
                    _ => {}
                }
            }

            thread::sleep(Duration::from_millis(2));

            self.chip8.emulate_cycle();
            if self.chip8.get_draw_flag() {
                self.draw_graphics(&mut canvas);
            }
        }

        Ok(())
    }

    fn draw_graphics(&self, canvas: &mut Canvas<Window>) {
        for row in 0..chip8::HEIGHT {
            for col in 0..chip8::WIDTH {
                let color = if self.chip8.is_set(row, col) {
                    WHITE
                } else {
                    BLACK
                };
                canvas.set_draw_color(color);
                let rectangle = Rect::new(
                    (col * SCALE as usize) as i32,
                    (row * SCALE as usize) as i32,
                    SCALE,
                    SCALE,
                );
                canvas.fill_rect(rectangle).unwrap_or_else(|_| {
                    panic!("failed to draw rect at row {}, column {}", row, col)
                })
            }
        }
        canvas.present();
    }
}

fn key_to_code(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None
    }
}