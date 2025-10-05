#![windows_subsystem = "windows"]
#![feature(maybe_uninit_slice)]
mod audio;
mod gui;
mod synth;
mod common;

const FRAME_RATE: usize = 60;
const SCREEN_WIDTH: u32 = 1260;
const SCREEN_HEIGHT: u32 = 800;

fn main() {
    //unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let sdl3_context = sdl3::init().expect("Failed to initialize sdl3");
    let video_subsystem = sdl3_context.video().expect("Failed to initialize video subsystem");
    
    let window = video_subsystem.window("Cav-Synth2", SCREEN_WIDTH, SCREEN_HEIGHT)
        .build()
        .expect("Failed to make window");
    let canvas = window.into_canvas();
    let event_pump = sdl3_context.event_pump().expect("Failed to initialize event pump");
    let texture_creator = canvas.texture_creator();

    let mut synth = synth::Synth::init(canvas, event_pump, & texture_creator);

    while !synth.should_quit() {
        let start = std::time::Instant::now();
        synth.update();
        synth.render().unwrap();
        std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / FRAME_RATE as f64).saturating_sub(start.elapsed()));
    }
}