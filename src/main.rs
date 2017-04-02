extern crate sdl2;

mod phi;
mod views;

use phi::{Events, Phi, View, ViewAction};

fn main() {
    println!("Hello, world!");

    ::phi::spawn("ArcadeRS Shooter",
                 |phi| Box::new(::views::main_menu::MainMenuView::new(phi)));
}
