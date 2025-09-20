#![windows_subsystem = "windows"]

use sdl3::{event::Event, keyboard::Keycode, pixels::Color, render::{Canvas, FRect}, video::{Window, WindowPos}};

mod audio;
mod synth;

fn main() {
    //unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let sdl3_context = sdl3::init().expect("Failed to initialize sdl3");
    let video_subsystem = sdl3_context.video().expect("Failed to initialize video subsystem");
    
    let window = video_subsystem.window("Hello", 1920, 1080)
        .borderless()
        .build()
        .expect("Failed to make window");
    let mut canvas = window.into_canvas();

    let mut event_pump = sdl3_context.event_pump().expect("Failed to initialize event pump");

    let mut window_grabbed = false;
    let window_pos = canvas.window().position();
    let mut window_pos = (window_pos.0 as f32, window_pos.1 as f32);

    let synth = synth::Synth::init();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseButtonDown { x, y, .. }
                if (x < 10.0 || x > 1910.0) || (y < 10.0 || y > 1070.0) => window_grabbed = true,
                Event::MouseButtonUp { .. } => window_grabbed = false,
                Event::MouseMotion { x, y, .. } 
                if window_grabbed => {
                    let (w_x, w_y) = canvas.window().position();
                    window_pos = (w_x as f32 + x, w_y as f32 + y);
                    canvas.window_mut().set_position(WindowPos::Positioned(window_pos.0 as i32), WindowPos::Positioned(window_pos.1 as i32));
                },
                _ => {},
            }
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}