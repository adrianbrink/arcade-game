extern crate sdl2;

mod phi;
mod views;

use sdl2::pixels::Color;
use phi::Events;

fn main() {
    println!("Hello, world!");

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video
        .window("ArcadeRS Shooter", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().accelerated().build().unwrap();

    let mut events = Events::new(sdl_context.event_pump().unwrap());

    loop {
        events.pump();

        if events.now.key_up == Some(true) {
            println!("Key up works");
            continue;
        }

        if events.now.quit || events.now.key_escape == Some(true) {
            break;
        }
        renderer.set_draw_color(Color::RGB(100, 0, 0));
        renderer.clear();
        renderer.present();
    }
}