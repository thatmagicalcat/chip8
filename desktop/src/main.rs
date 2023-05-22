use std::env;
use std::fs::File;
use std::io::Read;

mod vars;
use sdl2::keyboard::Keycode;
use vars::*;

use chip8_core::*;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        std::process::exit(1);
    }

    let mut chip8 = Emulator::new();
    let mut rom = File::open(&args[1]).expect("failed to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).expect("failed to read file");

    chip8.load(&buffer);

    let sdl_context = sdl2::init().expect("failed to initialize sdl");
    let video_subs = sdl_context.video().unwrap();
    let window = video_subs
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        for evt in event.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_btn(key) {
                        chip8.keypress(k, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_btn(key) {
                        chip8.keypress(k, false);
                    }
                }

                _ => {}
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);
    }
}

fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}


fn key_to_btn(key: Keycode) -> Option<usize> {
    match key {
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

        _ => None,
    }
}
