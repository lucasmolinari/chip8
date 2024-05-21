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
use std::time::Duration;

const SCALE: u32 = 16;

pub fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).ok_or("Expected a file path")?;
    let pb = Path::new(path).canonicalize().map_err(|e| e.to_string())?;

    let mut chip8 = Chip8::new();
    chip8.load(pb)?;

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
        chip8.fetch()?;
        chip8.execute()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        draw(&chip8, &mut canvas)?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    canvas.present();
    Ok(())
}

fn draw(chip: &Chip8, canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.present();
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
