/*
 *   Copyright (c) 2020 Ludwig Bogsveen
 *   All rights reserved.

 *   Permission is hereby granted, free of charge, to any person obtaining a copy
 *   of this software and associated documentation files (the "Software"), to deal
 *   in the Software without restriction, including without limitation the rights
 *   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *   copies of the Software, and to permit persons to whom the Software is
 *   furnished to do so, subject to the following conditions:
 
 *   The above copyright notice and this permission notice shall be included in all
 *   copies or substantial portions of the Software.
 
 *   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 *   SOFTWARE.
 */
use std::vec::Vec;

use rand::prelude::random;

use noise::Perlin;
use noise::NoiseFn;

use engine::window::Window;
use engine::core;
use engine::gfx::graphics::Graphics;
use engine::gfx::renderer::std_renderer::*;

fn rand_range(min: f32, max: f32) -> f32 {
    (random::<f32>() * (max - min)) + min
}

struct Vec4f {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

struct Vec3f {
    x: f32,
    y: f32,
    z: f32,
}

struct Vec2f {
    x: f32,
    y: f32,
}

impl Vec2f {
    fn set_mag(&mut self, mag: f32) {

        if self.x == 0.0 && self.y == 0.0 {
            return;
        }

        let scale = (mag * mag) / (self.x * self.x + self.y * self.y);
        self.x *= scale;
        self.y *= scale;
    }
}

struct Particle {
    bounds: Vec4f,
    vel: Vec2f,
}

impl Particle {
    fn new(win: &Window) -> Particle {
        Particle {
            bounds: Vec4f {
                x: rand_range(-1.0, 1.0),
				y: rand_range(-1.0, 1.0),
				z: 2.0 / win.get_width() as f32,
				w: 2.0 / win.get_height() as f32
            },
            vel: Vec2f {
                x: 0.0,
                y: 0.0,
            }
        }
    }
}

struct FlowField {
    win: Window,
    gfx: Graphics,
    noise: Perlin,

    noise_width: usize,
    noise_height: usize,

    max_particles: usize,

    increment: Vec3f,
    multiplier: f32,
    speed: f32,
    fixed_time_step: Option<f32>,

    n_new_particles: usize,

    particles: Vec<Particle>,
	index: usize,

	flow_field: Vec<f32>,
	z_off: f64,
}

impl FlowField {
    fn new(
        win: Window,
        noise_width: usize,
        noise_height: usize,
        max_particles: usize,
        multiplier: f32,
        increment: Vec3f,
        speed: f32,
        n_new_particles: usize,
        fixed_time_step: Option<f32>,
    ) -> FlowField {
        let mut win = win;

        let mut flow_field = Vec::with_capacity((noise_width+1)*(noise_height+1));
        for _ in 0..flow_field.capacity() {
            flow_field.push(0.0);
        }

        let mut particles = Vec::new();

        for _ in 0..max_particles {
			particles.push(Particle::new(&win));
		}

        FlowField {
            gfx: Graphics::new(&mut win),
            win,
            noise: Perlin::new(),
            noise_width,
            noise_height,
            max_particles,
            increment,
            multiplier,
            speed,
            n_new_particles,
            particles,
            index: 0,
            flow_field,
            z_off: 0.0,
            fixed_time_step,
        }
    }
}

impl core::Game for FlowField {
    fn update(&mut self, dt: f32, _fps: u32) -> bool {
        let ts;
        match self.fixed_time_step {
            Some(time_step) => {
                ts = time_step;
            }
            None => {
                ts = dt;
            }
        }        

        for y in 0..self.noise_height {
            for x in 0..self.noise_width {
                self.flow_field[x+y*self.noise_width] = (self.noise.get([x as f64 * self.increment.x as f64, y as f64 * self.increment.y as f64, self.z_off]) as f32 + 1.0) * self.multiplier;
            }
        }

        for particle in &mut self.particles {
			if particle.bounds.x < -1.0 { particle.bounds.x = 1.0 }
			if particle.bounds.y < -1.0 { particle.bounds.y = 1.0 }
            if particle.bounds.x > 1.0 { particle.bounds.x = -1.0 }
			if particle.bounds.y > 1.0 { particle.bounds.y = -1.0 }

			let x = (((particle.bounds.x + 1.0) / 2.0) * self.noise_width as f32) as usize;
			let y = (((particle.bounds.y + 1.0) / 2.0) * self.noise_height as f32) as usize;
			let flow = self.flow_field[x+y*self.noise_width];
			particle.vel.x += (flow.cos() * ts) / 10.0;
            particle.vel.y += (flow.sin() * ts) / 10.0;

            particle.vel.set_mag(1.0 / self.win.get_height() as f32 * self.speed);

            particle.bounds.x += particle.vel.x;
            particle.bounds.y += particle.vel.y;
		}

		self.z_off += ts as f64 * self.increment.z as f64;

        //////////////////////RENDER/////////////////////
        for _ in 0..self.n_new_particles {
			self.particles[self.index % self.max_particles] = Particle::new(&self.win);
			self.index += 1;
		}

		for particle in &self.particles {
            self.gfx.set_color(
                (particle.bounds.x + 1.0) / 2.0 * 0.05,
                (particle.bounds.y + 1.0) / 2.0 * 0.05,
                (1.0 - (particle.bounds.x + 1.0) / 2.0) * 0.05,
                1.0);

			self.gfx.fill_rect(particle.bounds.x, particle.bounds.y, particle.bounds.z, particle.bounds.w);
        }

		self.gfx.flush();
        self.gfx.update();
        self.win.poll_events();
        self.win.swap_buffers();
        !self.win.should_close()
    }
}

// Några saker man kanske vill testa är
// -m 6.28 --particlesPerUpdate 0 --noiseWidth 80 --noiseHeight 80 -i 0.1 -s 4.0
// -m 6.28 --particlesPerUpdate 110 --noiseWidth 20 --noiseHeight 20 -i 0.05 -z 0.00  -s 4.6


//Tusen tack ma boi Johannes för clap koden!
use clap::{App, Arg};

fn main() {
    let mut screen_w: u32 = 1920;
    let mut screen_h: u32 = 1080;
    let mut noise_w: u32 = 1200;
    let mut noise_h: u32 = 800;
    let mut particles: u32 = 10000;
    let mut increment: Vec3f = Vec3f {x: 0.01, y: 0.01, z: 0.01};
    let mut multiplier: f32 = 3.14 * 20.0;
    let mut speed: f32 = 4.0;
    let mut n_new_particles: u32 = 0;
    let mut fixed_time_step: Option::<f32> = Some(0.03);

    let flags = App::new("flowfield")
        .version("0.1.0")
        .author("Ludwig Bogsveen, <github.com/romptroll>")
        .about("a flowfield")
        .arg(
            Arg::with_name("screen width")
                .short("w")
                .long("screenWidth")
                .value_name("screen_w")
                .takes_value(true)
                .help("sets the screen width")
        )
        .arg(
            Arg::with_name("screen height")
                .short("h")
                .long("screenHeight")
                .value_name("screen_h")
                .takes_value(true)
                .help("sets the screen height")
        )
        .arg(
            Arg::with_name("noise width")
                .short("W")
                .long("noiseWidth")
                .value_name("noise_w")
                .takes_value(true)
                .help("sets the noise width")
        )
        .arg(
            Arg::with_name("noise height")
                .short("H")
                .long("noiseHeight")
                .value_name("noise_h")
                .takes_value(true)
                .help("sets the noise height")
        )
        .arg(
            Arg::with_name("particles")
                .short("p")
                .long("particles")
                .value_name("particles")
                .takes_value(true)
                .help("sets the amount of particles")
        )
        .arg(
            Arg::with_name("new particles")
                .short("n")
                .long("particlesPerUpdate")
                .value_name("new_particles")
                .takes_value(true)
                .help("new particles per update")
        )
        .arg(
            Arg::with_name("increment")
                .short("i")
                .long("increment")
                .value_name("increment")
                .takes_value(true)
                .help("Increments the flow field values each iteration. Larger increments leads to more random flowfields!")
        )
        .arg(
            Arg::with_name("increment x")
                .short("x")
                .long("incrementX")
                .value_name("increment x")
                .takes_value(true)
                .help("Increments the flow field values each iteration. Larger increments leads to more random flowfields!")
        )
        .arg(
            Arg::with_name("increment y")
                .short("y")
                .long("incrementY")
                .value_name("increment y")
                .takes_value(true)
                .help("Increments the flow field values each iteration. Larger increments leads to more random flowfields!")
        )
        .arg(
            Arg::with_name("increment z")
                .short("z")
                .long("incrementZ")
                .value_name("increment z")
                .takes_value(true)
                .help("Increments the flow field values each iteration. Larger increments leads to more random flowfields!")
        )
        .arg(
            Arg::with_name("multiplier")
                .short("m")
                .long("multiplier")
                .value_name("multiplier")
                .takes_value(true)
                .help("Multiplies all the flow field values with itself. Note that the values in the flowfield are angles in radians!")
        )
        .arg(
            Arg::with_name("speed")
                .short("s")
                .long("speed")
                .value_name("speed")
                .takes_value(true)
                .help("The speed in pixels which the particles travels each tick!")
        )
        .arg(
            Arg::with_name("fixed time step")
                .short("t")
                .long("timeStep")
                .value_name("fixed time step")
                .takes_value(true)
                .help("Enables you to pick a custom timestep. Ignore this if you don't want fixed time step!")
        )
        .get_matches();

    if flags.is_present("screen width") {
        screen_w = flags.value_of("screen width").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("screen height") {
        screen_h = flags.value_of("screen height").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("noise width") {
        noise_w = flags.value_of("noise width").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("noise height") {
        noise_h = flags.value_of("noise height").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("particles") {
        particles = flags.value_of("particles").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("new particles") {
        n_new_particles = flags.value_of("new particles").unwrap().parse::<u32>().unwrap();
    }
    if flags.is_present("increment") {
        let inc = flags.value_of("increment").unwrap().parse::<f32>().unwrap();
        increment.x = inc;
        increment.y = inc;
        increment.z = inc;
    }
    if flags.is_present("increment x") {
        increment.x = flags.value_of("increment x").unwrap().parse::<f32>().unwrap();
    }
    if flags.is_present("increment y") {
        increment.y = flags.value_of("increment y").unwrap().parse::<f32>().unwrap();
    }
    if flags.is_present("increment z") {
        increment.z = flags.value_of("increment z").unwrap().parse::<f32>().unwrap();
    }
    if flags.is_present("multiplier") {
        multiplier = flags.value_of("multiplier").unwrap().parse::<f32>().unwrap();
    }
    if flags.is_present("speed") {
        speed = flags.value_of("speed").unwrap().parse::<f32>().unwrap();
    }
    if flags.is_present("fixed time step") {
        fixed_time_step = Some(flags.value_of("fixed time step").unwrap().parse::<f32>().unwrap());
    }


    let mut win = Window::new(screen_w, screen_h, "").unwrap();
    win.make_current();

    core::init_gl(&mut win);

    unsafe { enable(Capability::Blending); }
    unsafe { blend_func(BlendMode::One, BlendMode::One); }

    core::init_game(FlowField::new(
        win,
        noise_w as usize,
        noise_h as usize, 
        particles as usize, 
        multiplier, 
        increment, 
        speed,
        n_new_particles as usize,
        fixed_time_step,
    ));
}