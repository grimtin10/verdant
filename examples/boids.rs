use std::{ops::Range, time::Instant};

use fastrand::f32;
use verdant::prelude::*;
use verdant::vec::Vec2;

// all the parameters that control the simulation
// boid settings
const BOID_COUNT: usize = 1000;
const BOID_SIZE: f32 = 10.;

// distances
const NEIGHBORHOOD: f32 = BOID_SIZE * 10.;
const SEPARATION_DIST: f32 = BOID_SIZE * 3.5;

// rule weights
const SEPARATION_WEIGHT: f32 = 2.5;
const ALIGN_WEIGHT: f32 = 1.0;
const COHESION_WEIGHT: f32 = 1.0;
const WANDER_WEIGHT: f32 = 2.5;

// constraints
const MAX_SPEED: f32 = 250.;
const MAX_FORCE: f32 = 3.;

fn random_range(range: Range<f32>) -> f32 {
    range.start + f32() * (range.start - range.end)
}

#[derive(Default)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
}

impl Boid {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::new(random_range(-MAX_SPEED..MAX_SPEED), random_range(-MAX_SPEED..MAX_SPEED)),
            ..Default::default()
        }
    }

    fn update(&mut self, dt: f32) {
        self.velocity += self.acceleration;

        if self.velocity.length() > MAX_SPEED {
            self.velocity /= self.velocity.length() / MAX_SPEED;
        }

        self.position += self.velocity * dt;
        self.acceleration = Vec2::ZERO;

        if self.position.x < -BOID_SIZE {
            self.position.x = 1920. + BOID_SIZE;
        }

        if self.position.y < -BOID_SIZE {
            self.position.y = 1080. + BOID_SIZE;
        }

        if self.position.x > 1920. + BOID_SIZE {
            self.position.x = -BOID_SIZE;
        }

        if self.position.y > 1080. + BOID_SIZE {
            self.position.y = -BOID_SIZE;
        }
    }
}

// we implement the Drawable traits for boids so we can just use `.draw(window)`
impl Drawable for Boid {
    fn draw_at(&self, window: &mut impl RenderSurface, x: f32, y: f32) {
        // we wrap it in `with_style` to avoid clobbering the state in the state machine
        // and we use `with_transform` to apply a transform to both the ellipse and the line
        window.with_style(|window| {
            window.with_transform(
                // the transform that will be applied to the draw commands
                Transform2d::rotation_rad(self.velocity.angle_rad())
                    .translate(x, y),

                // the draw commands
                |window| {
                    window.no_outline();
                    window.fill(Color::WHITE);
                    window.ellipse(0., 0., BOID_SIZE, BOID_SIZE);

                    window.outline(Color::BLACK, 1.);
                    window.line(0., 0., BOID_SIZE, 0.);
                }
            );
        });
    }

    fn draw(&self, window: &mut impl RenderSurface) {
        self.draw_at(window, self.position.x, self.position.y);
    }
}

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = WindowProperties::new("boids", 1920, 1080)
        .resizable(true)
        .build(&mut renderer);

    let mut boids = Vec::new();

    // initialize all the boids at a random position
    for _ in 0..BOID_COUNT {
        boids.push(Boid::new(Vec2::new(random_range(0f32..1920.), random_range(0f32..1080.))));
    }

    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    let mut last_time = Instant::now();

    while renderer.is_running() {
        // get the deltatime of the last frame for the FPS counter and updating the boids
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            window.background(Color::BLACK);

            // TODO: should probably optimize this loop so we can handle more boids
            //       O(n^2) is not the best for performance

            // calculate all the rules to apply to the boids
            // separation, alignment, and cohesion, plus a random wander vector
            for i in 0..boids.len() {
                let boid = &boids[i];

                let mut separation = Vec2::ZERO;
                let mut align = Vec2::ZERO;
                let mut cohesion = Vec2::ZERO;

                let mut neighbor_count = 0;
                let mut close_count = 0;
                for (j, other) in boids.iter().enumerate() {
                    if i != j {
                        let dist = boid.position.dist(other.position);

                        if dist < SEPARATION_DIST {
                            separation += (boid.position - other.position) / (dist * dist);
                            close_count += 1;
                        }

                        if dist < NEIGHBORHOOD {
                            align += other.velocity;
                            // scale to 0..1 for precision
                            cohesion += Vec2::new(other.position.x / 1920., other.position.y / 1080.);
                            neighbor_count += 1;
                        }
                    }
                }

                if close_count > 0 {
                    separation /= close_count as f32;
                    separation /= separation.length() / MAX_FORCE;
                }

                if neighbor_count > 0 {
                    align /= neighbor_count as f32;
                    cohesion /= neighbor_count as f32;

                    align = align.normalize() * MAX_SPEED - boid.velocity;
                    align /= align.length() / MAX_FORCE;

                    let toward_center = Vec2::new(cohesion.x * 1920., cohesion.y * 1080.) - boid.position;
                    cohesion = toward_center.normalize() * MAX_SPEED - boid.velocity;
                    cohesion /= cohesion.length() / MAX_FORCE;
                }

                let wander_mag = (MAX_FORCE * WANDER_WEIGHT).sqrt();

                boids[i].acceleration =
                    separation * SEPARATION_WEIGHT
                    + align * ALIGN_WEIGHT
                    + cohesion * COHESION_WEIGHT
                    + Vec2::new(
                        random_range(-wander_mag..wander_mag),
                        random_range(-wander_mag..wander_mag)
                    );
            }

            // update all the boids, applying "physics" to them
            for boid in &mut boids {
                boid.update(dt);
            }

            // draw the boids with a view set to crop
            // we put it in with_style to prevent the FPS text from being cropped off the screen
            window.with_style(|window| {
                window.set_view(1920., 1080., ViewMode::Crop);

                for boid in &boids {
                    boid.draw(window);
                }
            });

            window.fill(Color::GREEN);
            window.vertical_text_align(VerticalAlign::Top);
            window.text(&font, 0., 0., format!("FPS: {:.2}", 1. / dt));
        }

        renderer.flush()?;
    }

    Ok(())
}
