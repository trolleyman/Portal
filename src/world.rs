use prelude::*;

use entity::{Entity, Camera};
use sdl2::keyboard::{KeyboardState, Keycode, Mod};
use render::Render;

use cg::{self, Basis3};

#[derive(Clone)]
pub struct World {
	/// The camera information
	pub camera: Camera,
	/// All of the entities in the world.
	pub entities: Vec<Entity>,
}

impl World {
	pub fn new(cam: Camera) -> World {
		World {
			camera: cam,
			entities: Vec::new()
		}
	}
	
	pub fn tick(&mut self, dt: DT, state: &KeyboardState) {
		for ent in self.entities.iter_mut() {
			ent.tick(dt);
		}
		
		let speed = 0.5;
		let xrot = self.camera.xrot;
		if state.is_scancode_pressed(Scan::W) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new(0.0, 0.0, -speed).mul_s(dt)));
		}
		if state.is_scancode_pressed(Scan::S) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new(0.0, 0.0,  speed).mul_s(dt)));
		}
		if state.is_scancode_pressed(Scan::A) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new(-speed, 0.0, 0.0).mul_s(dt)));
		}
		if state.is_scancode_pressed(Scan::D) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new( speed, 0.0, 0.0).mul_s(dt)));
		}
		if state.is_scancode_pressed(Scan::Q) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new(0.0,  speed, 0.0).mul_s(dt)));
		}
		if state.is_scancode_pressed(Scan::E) {
			self.camera.translate(Basis3::from_angle_y(-cg::rad(xrot)).rotate_vector(&Vec3::new(0.0, -speed, 0.0).mul_s(dt)));
		}
	}
	
	pub fn render(&self, ren: &mut Render) {
		ren.set_camera(&self.camera);
		for ent in self.entities.iter() {
			ent.render(ren);
		}
	}
	
	pub fn handle_keydown(&mut self, key: &Keycode, keymod: &Mod, repeat: bool) {
		let (_, _, _) = (key, keymod, repeat);
	}
	
	pub fn handle_keyup(&mut self, key: &Keycode, keymod: &Mod) {
		let (_, _) = (key, keymod);
	}
	
	pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
		self.camera.rotate(x as f32 * 0.1, y as f32 * 0.1);
	}
	
	pub fn print(&self) {
		println!("x:{:.4}, y:{:.4}, z:{:.4}", self.camera.pos.x, self.camera.pos.y, self.camera.pos.z);
	}
}
