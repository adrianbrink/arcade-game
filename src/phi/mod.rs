#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use sdl2::render::Renderer;
use self::gfx::Sprite;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::path::Path;
use sdl2::ttf::Sdl2TtfContext;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space
    },
    else : {
        quit: Quit { .. }
    }
}

pub struct Phi<'a> {
    pub events: Events,
    pub renderer: Renderer<'a>,
    cached_fonts: HashMap<(&'static str, u16), ::sdl2::ttf::Font<'static, 'static>>,
    font_context: Sdl2TtfContext,
}

impl<'a> Phi<'a> {
    pub fn new(events: Events, renderer: Renderer<'a>, font_context: Sdl2TtfContext) -> Phi<'a> {
        Phi {
            events: events,
            renderer: renderer,
            cached_fonts: HashMap::new(),
            font_context: font_context,
        }
    }

    pub fn output_size(&self) -> (f64, f64) {
        let (w, h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }

    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: u16, color: Color) -> Option<Sprite> {
        self.font_context.load_font(Path::new(font_path), size).ok()
            .and_then(|font| font
                      .render(text)
                      .blended(color).ok()
                      .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
                      .map(Sprite::new))
    }
}

pub enum ViewAction {
    None,
    Quit,
    ChangeView(Box<View>),
}

pub trait View {
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}

pub fn spawn<F>(title: &str, init: F)
    where F: Fn(&mut Phi) -> Box<View>
{
    let sdl_context = ::sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let _image_context = ::sdl2::image::init(::sdl2::image::INIT_PNG).unwrap();
    let ttf_context = ::sdl2::ttf::init().unwrap();

    let window = video
        .window(title, 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut context = Phi::new(Events::new(sdl_context.event_pump().unwrap()),
                               window.renderer().accelerated().build().unwrap(),
                               ttf_context
                      );

    let mut current_view = init(&mut context);

    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    loop {
        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        context.events.pump(&mut context.renderer);

        match current_view.render(&mut context, elapsed) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,
            ViewAction::ChangeView(new_view) => current_view = new_view,
        }
    }
}
