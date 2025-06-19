use wasm_bindgen::prelude::*;
use std::f32::consts::PI;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct ParticleSimulation {
    particles: Vec<Particle>,
    force_table: Vec<f32>,
    radio_by_type: Vec<f32>,
    params: SimParams,
    canvas_width: f32,
    canvas_height: f32,
}

#[derive(Clone, Copy)]
struct Particle {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    acc_x: f32,
    acc_y: f32,
    ptype: u32,
}

#[derive(Clone, Copy)]
struct SimParams {
    radius: f32,
    delta_t: f32,
    friction: f32,
    repulsion: f32,
    attraction: f32,
    k: f32,
    balance: f32,
    num_particle_types: u32,
    ratio: f32,
    force_multiplier: f32,
    max_expected_neighbors: f32,
}

#[wasm_bindgen]
impl ParticleSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(particle_count: usize, num_types: u32, canvas_width: f32, canvas_height: f32) -> ParticleSimulation {
        console_log!("Initializing Rust WASM particle simulation with {} particles", particle_count);
        
        let mut particles = Vec::with_capacity(particle_count);
        for _ in 0..particle_count {
            particles.push(Particle {
                pos_x: js_sys::Math::random() as f32 * canvas_width,
                pos_y: js_sys::Math::random() as f32 * canvas_height,
                vel_x: 0.0,
                vel_y: 0.0,
                acc_x: 0.0,
                acc_y: 0.0,
                ptype: (js_sys::Math::random() * num_types as f64) as u32,
            });
        }

        let force_table = vec![0.0; (num_types * num_types) as usize];
        let radio_by_type = vec![0.0; num_types as usize];

        ParticleSimulation {
            particles,
            force_table,
            radio_by_type,
            params: SimParams {
                radius: 50.0,
                delta_t: 0.22,
                friction: 0.71,
                repulsion: 50.0,
                attraction: 0.62,
                k: 16.57,
                balance: 0.79,
                num_particle_types: num_types,
                ratio: 0.0,
                force_multiplier: 2.33,
                max_expected_neighbors: 400.0,
            },
            canvas_width,
            canvas_height,
        }
    }

    #[wasm_bindgen]
    pub fn update_params(&mut self, 
        radius: f32, delta_t: f32, friction: f32, repulsion: f32, 
        attraction: f32, k: f32, balance: f32, ratio: f32, 
        force_multiplier: f32, max_expected_neighbors: f32) {
        self.params.radius = radius;
        self.params.delta_t = delta_t;
        self.params.friction = friction;
        self.params.repulsion = repulsion;
        self.params.attraction = attraction;
        self.params.k = k;
        self.params.balance = balance;
        self.params.ratio = ratio;
        self.params.force_multiplier = force_multiplier;
        self.params.max_expected_neighbors = max_expected_neighbors;
    }

    #[wasm_bindgen]
    pub fn update_force_table(&mut self, force_table: &[f32]) {
        self.force_table.clear();
        self.force_table.extend_from_slice(force_table);
    }

    #[wasm_bindgen]
    pub fn update_radio_by_type(&mut self, radio_by_type: &[f32]) {
        self.radio_by_type.clear();
        self.radio_by_type.extend_from_slice(radio_by_type);
    }

    #[wasm_bindgen]
    pub fn simulate_step(&mut self) {
        let particle_count = self.particles.len();
        
        // Reset accelerations
        for particle in &mut self.particles {
            particle.acc_x = 0.0;
            particle.acc_y = 0.0;
        }

        // Calculate forces between particles
        for i in 0..particle_count {
            let mut force_x = 0.0;
            let mut force_y = 0.0;
            let mut neighbors_count = 0;

            let particle_i = self.particles[i];
            let my_type = particle_i.ptype as usize;
            let effective_radius = self.params.radius + 
                (self.params.radius * self.radio_by_type[my_type] * self.params.ratio);

            for j in 0..particle_count {
                if i == j { continue; }

                let particle_j = self.particles[j];
                
                // Calculate distance with wrapping
                let mut dir_x = particle_j.pos_x - particle_i.pos_x;
                let mut dir_y = particle_j.pos_y - particle_i.pos_y;

                // Wrap around canvas boundaries
                dir_x = dir_x - (dir_x / self.canvas_width + 0.5).floor() * self.canvas_width;
                dir_y = dir_y - (dir_y / self.canvas_height + 0.5).floor() * self.canvas_height;

                let dist_sq = dir_x * dir_x + dir_y * dir_y;
                let dist = dist_sq.sqrt();

                if dist == 0.0 || dist > effective_radius { continue; }

                neighbors_count += 1;

                let r = dist / effective_radius;
                let force_table_index = (particle_i.ptype * self.params.num_particle_types + particle_j.ptype) as usize;
                let a = self.force_table[force_table_index];

                let rep_decay = r * self.params.k;
                let repulsion = self.params.repulsion * (1.0 / (1.0 + rep_decay * rep_decay));
                let attraction = self.params.attraction * r * r;
                let f = a * (repulsion - attraction) * self.params.force_multiplier;

                let inv_dist = 1.0 / dist;
                force_x += dir_x * inv_dist * f;
                force_y += dir_y * inv_dist * f;
            }

            // Apply adaptive force multiplier based on density
            let normalized_density = (neighbors_count as f32) / self.params.max_expected_neighbors;
            let clamped_density = normalized_density.min(1.0).max(0.0);

            let min_force_mult = 1.0 + (0.01 - 1.0) * self.params.balance;
            let max_force_mult = 1.0 + (4.0 - 1.0) * self.params.balance;
            let adaptive_multiplier = max_force_mult + (min_force_mult - max_force_mult) * clamped_density;

            // Update velocity and position
            let mut vel_x = particle_i.vel_x * self.params.friction;
            let mut vel_y = particle_i.vel_y * self.params.friction;
            
            vel_x += force_x * self.params.delta_t * adaptive_multiplier;
            vel_y += force_y * self.params.delta_t * adaptive_multiplier;

            let mut pos_x = particle_i.pos_x + vel_x * self.params.delta_t;
            let mut pos_y = particle_i.pos_y + vel_y * self.params.delta_t;

            // Wrap positions
            pos_x = pos_x - self.canvas_width * (pos_x / self.canvas_width).floor();
            pos_y = pos_y - self.canvas_height * (pos_y / self.canvas_height).floor();

            if pos_x < 0.0 { pos_x += self.canvas_width; }
            if pos_y < 0.0 { pos_y += self.canvas_height; }

            self.particles[i].pos_x = pos_x;
            self.particles[i].pos_y = pos_y;
            self.particles[i].vel_x = vel_x;
            self.particles[i].vel_y = vel_y;
        }
    }

    #[wasm_bindgen]
    pub fn get_particle_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.particles.len() * 8);
        for particle in &self.particles {
            data.push(particle.pos_x);
            data.push(particle.pos_y);
            data.push(particle.vel_x);
            data.push(particle.vel_y);
            data.push(particle.acc_x);
            data.push(particle.acc_y);
            data.push(particle.ptype as f32);
            data.push(0.0); // padding
        }
        data
    }

    #[wasm_bindgen]
    pub fn reset_particles(&mut self) {
        for particle in &mut self.particles {
            particle.pos_x = js_sys::Math::random() as f32 * self.canvas_width;
            particle.pos_y = js_sys::Math::random() as f32 * self.canvas_height;
            particle.vel_x = 0.0;
            particle.vel_y = 0.0;
            particle.acc_x = 0.0;
            particle.acc_y = 0.0;
            particle.ptype = (js_sys::Math::random() * self.params.num_particle_types as f64) as u32;
        }
    }

    #[wasm_bindgen]
    pub fn set_canvas_size(&mut self, width: f32, height: f32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }
}
