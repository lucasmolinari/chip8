mod chip8;

use chip8::Chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::path::Path;

const SCALE: u32 = 16;

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {}
    let path = args.get(1).ok_or("Expected a file path")?;
    let rom = Path::new(path).canonicalize().map_err(|e| e.to_string())?;

    let mut chip8 = Chip8::new();
    chip8.load(rom)?;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CHIP8", 64 * SCALE, 32 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        chip8.tick()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = mapkey(key) {
                        chip8.keypress(k, 1);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = mapkey(key) {
                        chip8.keypress(k, 0);
                    }
                }
                _ => {}
            }
        }
        draw(&chip8, &mut canvas)?;
    }

    Ok(())
}

fn draw(chip: &Chip8, canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in chip.get_display().iter().enumerate() {
        if *pixel == 1 {
            let x = (i % 64) as i32;
            let y = (i / 64) as i32;

            let rect = Rect::new(x * SCALE as i32, y * SCALE as i32, SCALE, SCALE);
            canvas.fill_rect(rect)?;
        }
    }
    canvas.present();
    Ok(())
}

fn mapkey(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5), // Keyboard                   Chip-8
        Keycode::E => Some(0x6), // +---+---+---+---+          +---+---+---+---+
        Keycode::R => Some(0xD), // | 1 | 2 | 3 | 4 |          | 1 | 2 | 3 | C |
        Keycode::A => Some(0x7), // +---+---+---+---+          +---+---+---+---+
        Keycode::S => Some(0x8), // | Q | W | E | R |          | 4 | 5 | 6 | D |
        Keycode::D => Some(0x9), // +---+---+---+---+    =>    +---+---+---+---+
        Keycode::F => Some(0xE), // | A | S | D | F |          | 7 | 8 | 9 | E |
        Keycode::Z => Some(0xA), // +---+---+---+---+          +---+---+---+---+
        Keycode::X => Some(0x0), // | Z | X | C | V |          | A | 0 | B | F |
        Keycode::C => Some(0xB), // +---+---+---+---+          +---+---+---+---+
        Keycode::V => Some(0xF),
        _ => None,
    }
}
