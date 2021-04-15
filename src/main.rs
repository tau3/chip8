use crate::chip8::Chip8;
use crate::ui::Presenter;

mod buffer;
mod chip8;
mod ui;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_game("invaders.c8");

    let mut presenter = Presenter::new(chip8);
    presenter.event_loop().expect("breakage");
}
