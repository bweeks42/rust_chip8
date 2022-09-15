extern crate piston_window;
mod core;
mod instruction;

use graphics::{Graphics, Context};
use opengl_graphics::{OpenGL, GlGraphics};
use piston::{EventSettings, Events, WindowSettings, RenderEvent, RenderArgs, Button, PressEvent, Key, AdvancedWindow, ReleaseEvent};
use piston_window::PistonWindow;

use crate::core::{DISPLAY_HEIGHT, DISPLAY_WIDTH, CPU};
use std::{fs::read, time::{SystemTime, Duration}};

const IPS: u64 = 150;

fn main() {
    let mut cpu = CPU::new();

    // Get rom
    let rom = read("/Users/bweeks/code/rust_chip8/roms/tetris.ch8").unwrap(); 
    cpu.load(rom);

    // Setup graphics
    let mut event_settings = EventSettings::new();
    event_settings.max_fps = 60;
    event_settings.ups = IPS;
    let mut events = Events::new(event_settings);
    
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new(
        "CHIP-8",
        [640 , 320]
    ).exit_on_esc(true)
    .graphics_api(opengl)
    .build()
    .unwrap();
    let mut gl = GlGraphics::new(opengl);
    let mut running = true; 
    let mut last_tick = SystemTime::now();
    let delay_duration = Duration::new(0, 160000000);
    
    // Event loop
    while let Some(e) = events.next(&mut window) {
        if running {
            cpu.step();
        }

        // Delay
        let now = SystemTime::now();
        if now.duration_since(last_tick).ok().unwrap() > delay_duration {
            last_tick = now;
            if cpu.delay_timer > 0 {
                cpu.delay_timer -= 1;
            }
        }
        
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                draw_screen(args, c, g, &cpu.display);
            });
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            let key_hex = hex_for_key(key);
            if key_hex >= 0 {
                let key_index = key_hex as usize;
                cpu.keys[key_index] = 1;
            }

            // Debug keys
            if key == Key::Space {
                running = !running;
            }

            if key == Key::P {
                cpu.dump_registers();
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            let key_hex = hex_for_key(key);
            if key_hex >= 0 {
                let key_index = key_hex as usize;
                cpu.keys[key_index] = 0;
            }
        }
    }
}

fn hex_for_key(key: Key) -> i32 {
    match key {
        Key::D1 => {0x1},
        Key::D2 => {0x2},
        Key::D3 => {0x3},
        Key::D4 => {0xC},
        Key::Q => {0x4},
        Key::W => {0x5},
        Key::E => {0x6},
        Key::R => {0xD},
        Key::A => {0x7},
        Key::S => {0x8},
        Key::D => {0x9},
        Key::F => {0xE},
        Key::Z => {0xA},
        Key::X => {0x0},
        Key::C => {0xB},
        Key::V => {0xF}
        _ => {-1}
    }
}


fn draw_screen<G: Graphics>(args: RenderArgs, c: Context, g: &mut G, d: &[u8]) {
    use graphics::{clear, rectangle};
    clear([0.0, 0.0, 0.0, 1.0], g);
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let pix = d[y*DISPLAY_WIDTH + x];
            if pix > 0 {
                rectangle(
                    [1.0, 1.0, 1.0, 1.0], 
                    rectangle::square((x * 10) as f64, (y * 10) as f64, 9.0),
                    c.transform, 
                    g
                )
            }
        }
    }
}
