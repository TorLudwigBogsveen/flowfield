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

struct FlowField {
    win: Window,
    gfx: Graphics,
    noise: Perlin,

    noise_width: usize,
    noise_height: usize,

    max_particles: usize,

    multiplier: f32,

    n_new_particles: usize,

    particles: Vec<Vec4f>,
	index: usize,

	flow_field: Vec<f32>,
	z_off: f64,
}

impl FlowField {
    fn new(win: Window, noise_width: usize, noise_height: usize, max_particles: usize, multiplier: f32, n_new_particles: usize) -> FlowField {
        let mut win = win;

        let mut flow_field = Vec::with_capacity((noise_width+1)*(noise_height+1));
        for _ in 0..flow_field.capacity() {
            flow_field.push(0.0);
        }

        let mut particles = Vec::new();

        for _ in 0..max_particles {
			particles.push(Vec4f {
				x: rand_range(-1.0, 1.0),
				y: rand_range(-1.0, 1.0),
				z: 2.0 / win.get_width() as f32,
				w: 2.0 / win.get_height() as f32
            });
		}

        FlowField {
            gfx: Graphics::new(&mut win),
            win,
            noise: Perlin::new(),
            noise_width,
            noise_height,
            max_particles,
            multiplier,
            n_new_particles,
            particles,
            index: 0,
            flow_field,
            z_off: 0.0,
        }
    }
}

impl core::Game for FlowField {
    fn update(&mut self, dt: f32, fps: u32) -> bool {

        for y in 0..self.noise_height {
            for x in 0..self.noise_width {
                self.flow_field[x+y*self.noise_width] = (self.noise.get([x as f64 / 100.0, y as f64 / 100.0, self.z_off]) as f32 + 1.0) * self.multiplier;
            }
        }

        for particle in &mut self.particles {
			if particle.x < -1.0 { particle.x = 1.0 }
			if particle.y < -1.0 { particle.y = 1.0 }
            if particle.x > 1.0 { particle.x = -1.0 }
			if particle.y > 1.0 { particle.y = -1.0 }

			let x = (((particle.x + 1.0) / 2.0) * self.noise_width as f32) as usize;
			let y = (((particle.y + 1.0) / 2.0) * self.noise_height as f32) as usize;
			let flow = self.flow_field[x+y*self.noise_width];
			particle.x += (flow.cos() * dt) / 10.0;
			particle.y += (flow.sin() * dt) / 10.0;
		}

		self.z_off += dt as f64 / 100.0;

        //////////////////////RENDER/////////////////////
        for _ in 0..self.n_new_particles {
			self.particles[self.index % self.max_particles] = Vec4f {
				x: rand_range(-1.0, 1.0),
				y: rand_range(-1.0, 1.0),
				z: 2.0 / self.win.get_width() as f32,
				w: 2.0 / self.win.get_height() as f32
            };
			self.index += 1;
		}

		for particle in &self.particles {
            self.gfx.set_color((particle.x + 1.0) / 2.0 * 0.05, (particle.y + 1.0) / 2.0 * 0.05, (1.0 - (particle.x + 1.0) / 2.0) * 0.05, 1.0);
			self.gfx.fill_rect(particle.x, particle.y, particle.z, particle.w);
        }

		self.gfx.flush();
        self.gfx.update();
        self.win.poll_events();
        self.win.swap_buffers();
        !self.win.should_close()
    }
}

fn main() {
    let mut win = Window::new(1920, 1080, "du e gay").unwrap();
    win.make_current();

    core::init_gl(&mut win);

    unsafe { enable(Capability::Blending); }
    unsafe { blend_func(BlendMode::One, BlendMode::One); }

    core::init_game(FlowField::new(win, 1920, 1080, 10000, 3.14 * 0.5 * 1.6, 0));
}
