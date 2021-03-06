use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{Sprite, CopySprite, AnimatedSprite};
use std::path::Path;
use std::option::Option;
use sdl2::pixels::Color;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::image::LoadTexture;
use views::shared::BgSet;

const PLAYER_SPEED: f64 = 180.0;
const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;
const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;
const BULLET_SPEED: f64 = 240.0;
const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;
const DEBUG: bool = false;

#[derive(Clone, Copy)]
enum ShipFrame {
    UpNorm = 0,
    UpFast = 1,
    UpSlow = 2,
    MidNorm = 3,
    MidFast = 4,
    MidSlow = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8,
}

#[derive(Clone, Copy)]
struct RectBullet {
    rect: Rectangle,
}

struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
}

struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64,
}

pub struct ShipView {
    player: Ship,
    asteroid: Asteroid,
    bullets: Vec<RectBullet>,
    bg: BgSet,
}

impl Ship {
    fn spawn_bullets(&self) -> Vec<RectBullet> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + SHIP_H - 10.0;

        vec![
            RectBullet {
                rect: Rectangle {
                    x: cannons_x,
                    y: cannon1_y,
                    w: BULLET_W,
                    h: BULLET_H,
                }
            },
            RectBullet {
                rect: Rectangle {
                    x: cannons_x,
                    y: cannon2_y,
                    w: BULLET_W,
                    h: BULLET_H,
                }
            }
        ]
    }
}

impl RectBullet {
    fn update(mut self, phi: &mut Phi, dt: f64) -> Option<Self> {
        let (w, _) = phi.output_size();
        self.rect.x += BULLET_SPEED * dt;

        if self.rect.x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect.to_sdl());
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}

impl Asteroid {
    fn new(phi: &mut Phi) -> Asteroid {
        let mut asteroid = 
        Asteroid {
            sprite: Asteroid::get_sprite(phi, 1.0),
            rect: Rectangle {
                w: 0.0,
                h: 0.0,
                x: 0.0,
                y: 0.0,
            },
            vel: 0.0,
        };

        asteroid.reset(phi);
        asteroid
    }

    fn reset(&mut self, phi: &mut Phi) {
        let (w, h) = phi.output_size();

        self.sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        self.rect = Rectangle {
            w: ASTEROID_SIDE,
            h: ASTEROID_SIDE,
            x: w,
            y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
        };

        self.vel = ::rand::random::<f64>().abs() * 100.0 + 50.0;
    }

    fn update(&mut self, phi: &mut Phi, dt: f64) {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);
        if self.rect.x <= -ASTEROID_SIDE {
            self.reset(phi);
        }
    }

    fn render(&mut self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        w: ASTEROID_SIDE,
                        h: ASTEROID_SIDE,
                        x: ASTEROID_SIDE * xth as f64,
                        y: ASTEROID_SIDE * yth as f64,
                    }).unwrap());
            }
        }

        AnimatedSprite::with_fps(asteroid_sprites, fps)
    }
}

impl ShipView {
    pub fn new(phi: &mut Phi) -> ShipView {
        let bg = BgSet::new(&mut phi.renderer);
        ShipView::with_background(phi, bg)
    }

    pub fn with_background(phi: &mut Phi, bg: BgSet) -> ShipView {
        // Get the spaceship's sprites
        let spritesheet = Sprite::load(&mut phi.renderer, "assets/spaceship.png").unwrap();
        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: SHIP_W,
                    h: SHIP_H,
                    x: SHIP_W * x as f64,
                    y: SHIP_H * y as f64,
                }).unwrap());
            }
        }

        ShipView {
            player: Ship {
                rect: Rectangle {
                    x: 64.0,
                    y: 64.0,
                    w: SHIP_W,
                    h: SHIP_H,
                },
                sprites: sprites,
                current: ShipFrame::MidNorm,
            },

            bullets: vec![],

            asteroid: Asteroid::new(phi),

            bg: bg,
        }
    }
}

impl View for ShipView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
            return ViewAction::Quit;
        }

        let diagonal = (phi.events.key_up ^ phi.events.key_down) &&
                       (phi.events.key_left ^ phi.events.key_right);

        let moved = if diagonal { 1.0 / 2.0f64.sqrt() } else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match (phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match (phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        self.player.rect.x += dx;
        self.player.rect.y += dy;

        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 * 0.70,
            h: phi.output_size().1,
        };

        self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

        self.player.current = if dx == 0.0 && dy < 0.0 {
            ShipFrame::UpNorm
        } else if dx > 0.0 && dy < 0.0 {
            ShipFrame::UpFast
        } else if dx < 0.0 && dy < 0.0 {
            ShipFrame::UpSlow
        } else if dx == 0.0 && dy == 0.0 {
            ShipFrame::MidNorm
        } else if dx > 0.0 && dy == 0.0 {
            ShipFrame::MidFast
        } else if dx < 0.0 && dy == 0.0 {
            ShipFrame::MidSlow
        } else if dx == 0.0 && dy > 0.0 {
            ShipFrame::DownNorm
        } else if dx > 0.0 && dy > 0.0 {
            ShipFrame::DownFast
        } else if dx < 0.0 && dy > 0.0 {
            ShipFrame::DownSlow
        } else {
            unreachable!()
        };

        self.bullets =
            self.bullets.iter()
            .filter_map(|bullet| bullet.update(phi, elapsed))
            .collect();

        self.asteroid.update(phi, elapsed);

        if phi.events.now.key_space == Some(true) {
            self.bullets.append(&mut self.player.spawn_bullets());
        }

        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.player.rect.to_sdl());
        }

        phi.renderer.copy_sprite(&self.player.sprites[self.player.current as usize], self.player.rect);

        for bullet in &self.bullets {
            bullet.render(phi);
        }

        self.asteroid.render(phi);

        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}
