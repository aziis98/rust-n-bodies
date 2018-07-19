extern crate rand;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use rand::Rng;
use std::fmt;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

#[derive(PartialEq)]
struct Vector2f {
    x: f32,
    y: f32
}

impl fmt::Debug for Vector2f {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

impl Vector2f {

    fn new() -> Vector2f {
        Vector2f {
            x: 0.0,
            y: 0.0
        }
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

}

impl<'a> std::ops::Add<&'a Vector2f> for &'a Vector2f {
    type Output = Vector2f;

    fn add(self, other: &Vector2f) -> Vector2f {
        Vector2f {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl<'a> std::ops::Sub<&'a Vector2f> for &'a Vector2f {
    type Output = Vector2f;

    fn sub(self, other: &Vector2f) -> Vector2f {
        Vector2f {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl<'a> std::ops::Mul<&'a Vector2f> for f32 {
    type Output = Vector2f;

    fn mul(self, v: &Vector2f) -> Vector2f {
        Vector2f {
            x: self * v.x,
            y: self * v.y
        }
    }
}

#[derive(PartialEq, Debug)]
struct Particle {
    pos: Vector2f,
    vel: Vector2f,
    acc: Vector2f
}

impl Particle {
    fn new(pos: Vector2f, vel: Vector2f) -> Particle {
        Particle {
            pos,
            vel,
            acc: Vector2f::new()
        }
    }

    fn compute_force(a: &Particle, b: &Particle) -> Vector2f {
        let distance = (&b.pos - &a.pos).length().max(1.0); 
        let force = G_CONST / (distance * distance * distance);

        if force.is_normal() {
            force * &(&b.pos - &a.pos)
        }
        else {
            Vector2f::new()
        }
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    particles: Vec<Particle>
}

const WALL_BOUNCYNESS: f32 = 0.25;
const G_CONST : f32 = 10e2;
const PARTICLE_COUNT: u32 = 60;

const SIMULATION_ITERATIONS: u32 = 1;
const SIMULATION_SPEED: f32 = 1.0;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 900;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE:   [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let my_particles = &self.particles;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            for particle in my_particles {
                let circle_box = rectangle::centered_square(particle.pos.x.into(), particle.pos.y.into(), 2.5);
                ellipse(WHITE, circle_box, c.transform, gl);
            }

        });
    }

    fn update(&mut self, args: &UpdateArgs) {

        for _ in 0 .. SIMULATION_ITERATIONS {

            for mut p in &mut self.particles {
                (&mut p.acc).reset();
            }

            let particles = &mut self.particles;
            let count = particles.len();

            for i in 0 .. count {
                for j in 0 .. (i + 1) {
                    let acc = Particle::compute_force(&particles[i], &particles[j]);

                    particles[i].acc = &particles[i].acc + &acc;
                    particles[j].acc = &particles[j].acc - &acc;
                }
            }

            for p in particles {

                let comb_dt = args.dt as f32 * SIMULATION_SPEED / SIMULATION_ITERATIONS as f32;

                p.vel = &p.vel + &(comb_dt * &p.acc);
                p.pos = &p.pos + &(comb_dt * &p.vel);

                if p.pos.x < 0.0 {
                    p.pos.x = 0.0;
                    p.vel.x *= -WALL_BOUNCYNESS;
                }
                if p.pos.x > WIDTH as f32 {
                    p.pos.x = WIDTH as f32;
                    p.vel.x *= -WALL_BOUNCYNESS;
                }
                if p.pos.y < 0.0 {
                    p.pos.y = 0.0;
                    p.vel.y *= -WALL_BOUNCYNESS;
                }
                if p.pos.y > HEIGHT as f32 {
                    p.pos.y = HEIGHT as f32;
                    p.vel.y *= -WALL_BOUNCYNESS;
                }
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "N-Bodies Simulation",
            [WIDTH, HEIGHT]
        )
        .opengl(opengl)
        .resizable(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut rng = rand::thread_rng();
    
    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        particles: (1 .. PARTICLE_COUNT).map(|_i| {
            Particle::new(
                Vector2f { 
                    x: rng.gen::<f32>() * (WIDTH as f32),
                    y: rng.gen::<f32>() * (HEIGHT as f32) 
                },
                Vector2f { 
                    x: (rng.gen::<f32>() - 0.5) * 2.0 * 5.0,
                    y: (rng.gen::<f32>() - 0.5) * 2.0 * 5.0
                }
            )
        }).collect()
    };

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);

            // println!("");
            // for p in &app.particles {
            //     println!("{:?}", p);
            // }

        }
    }
}