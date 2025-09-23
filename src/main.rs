//#![windows_subsystem = "windows"]

use core::f32;
use image::Pixel;
use sdl3::{event::Event, keyboard::Keycode, pixels::{Color, FColor, PixelFormat, PixelFormatEnum, PixelMasks}, render::{Canvas, FPoint, FRect, Vertex, VertexIndices}, sys::pixels::{SDL_PIXELFORMAT_ABGR8888, SDL_PIXELFORMAT_RGBA8888}, video::{Window, WindowPos}};

mod audio;
mod gui;
mod synth;

const FRAME_RATE: usize = 60;

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
    let texture_creator = canvas.texture_creator();
    let knob = image::ImageReader::new(std::io::Cursor::new(include_bytes!("../assets/knob2.png")))
        .with_guessed_format().unwrap().decode().unwrap();
    let mut texture = texture_creator.create_texture_streaming(unsafe {PixelFormat::from_ll(SDL_PIXELFORMAT_ABGR8888)}, knob.width(), knob.height()).unwrap();
    let rgba = knob.to_rgba8();
    println!("{:?}", knob.color());
    texture.update(None, &rgba, 4 * knob.width() as usize).unwrap();
   // texture.update(rect, pixel_data, pitch)

    let synth = synth::Synth::init();
    let mut start = None;
    let mut end = None;
    let mut cables: Vec<(FPoint, FPoint)> = Vec::new();
    let mut mouse_down = false;
    let mut angle = 0.0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseButtonDown { x, y, .. }
                if (x < 10.0 || x > 1910.0) || (y < 10.0 || y > 1070.0) => window_grabbed = true,
                Event::MouseButtonDown { x, y, .. } => {
                    start = Some(FPoint::new(x, y));
                    end = Some(FPoint::new(x, y));
                    mouse_down = true;
                }
                Event::MouseButtonUp { .. } => {
                    match (start, end) {
                        (Some(start), Some(end)) => cables.push((start, end)),
                        _ => {},
                    }
                    window_grabbed = false;
                    mouse_down = false;
                },
                Event::MouseMotion { x, y, .. } 
                if window_grabbed => {
                    let (w_x, w_y) = canvas.window().position();
                    window_pos = (w_x as f32 + x, w_y as f32 + y);
                    canvas.window_mut().set_position(WindowPos::Positioned(window_pos.0 as i32), WindowPos::Positioned(window_pos.1 as i32));
                },
                Event::MouseMotion { x, y, xrel, .. }
                if mouse_down => {
                    if let Some(start) = start {
                        end = Some(FPoint::new(x, y));
                    }
                    angle = (angle + xrel as f64 / 10.0).clamp(0.0, 360.0);
                },
                _ => {},
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.copy(
            &texture,
            FRect::new(0.0, (angle / 360.0 * (knob.height() - knob.width()) as f64 / 64.0).floor() as f32 * 64.0, 64.0, 64.0),
            FRect::new(0.0, 0.0, 64.0, 64.0),
        ).unwrap();
        for &(start, end) in cables.iter() {
            draw_cable(&mut canvas, start, end, 8.0);
        }
        match (start, end) {
            (Some(start), Some(end)) => draw_cable(&mut canvas, start, end, 8.0),
            _ => {},
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / FRAME_RATE as f64));
    }
}

fn draw_cable(canvas: &mut Canvas<Window>, start: FPoint, end: FPoint, width: f32) {
    let angle = (end.y - start.y).atan2(end.x - start.x);
    let perp_ratio_cos = (angle - f32::consts::FRAC_PI_2).cos();
    let perp_ratio_sin = (angle - f32::consts::FRAC_PI_2).sin();
    let x_perp = width / 2.0 * perp_ratio_cos;
    let y_perp = width / 2.0 * perp_ratio_sin;
    let vertices = [
        new_vertex(FPoint::new(start.x - x_perp , start.y - y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x + x_perp, end.y + y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x - x_perp, end.y - y_perp), FColor::RGB(0.0, 0.4, 0.4), FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(start.x - x_perp, start.y - y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x + x_perp, end.y + y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(start.x + x_perp, start.y + y_perp), FColor::RGB(0.0, 0.4, 0.4), FPoint::new(1.0, 1.0)),
    ];
    let indices = VertexIndices::Sequential;
    canvas.render_geometry(&vertices, None, indices).unwrap();
}

fn new_vertex(position: FPoint, color: FColor, tex_coord: FPoint) -> Vertex {
    Vertex { position, color, tex_coord }
}