use prelude::*;

use entity::{Entity, Camera, Portal};
use sdl2::keyboard::{KeyboardState, Keycode, Mod};
use render::Render;

#[derive(Clone)]
pub struct World {
	/// The camera information
	pub camera: Camera,
	/// All of the entities in the world.
	pub entities: Vec<Entity>,
	portals: Option<(Portal, Portal)>,
}

impl World {
	pub fn new(cam: Camera) -> World {
		World {
			camera: cam,
			entities: Vec::new(),
			portals: None,
		}
	}
	
	pub fn set_portals(&mut self, p1: Portal, p2: Portal) {
		self.portals = Some((p1, p2));
	}
	pub fn get_portals(&self) -> Option<(Portal, Portal)> {
		self.portals
	}
	
	pub fn tick(&mut self, dt: DT, state: &KeyboardState) {
		for ent in self.entities.iter_mut() {
			ent.tick(dt);
		}
		
		let speed = if state.is_scancode_pressed(Scan::LShift) || state.is_scancode_pressed(Scan::RShift) {2.0}
		            else if state.is_scancode_pressed(Scan::LAlt) || state.is_scancode_pressed(Scan::RAlt) {0.1}
		            else {0.5};
		let dp = speed * dt;
		let rot = Rot3::new(Vec3::new(0.0, -self.camera.get_xrot(), 0.0));
		if state.is_scancode_pressed(Scan::W) {
			self.camera.translate(rot.rotate(&Vec3::new(0.0, 0.0,  dp)));
		}
		if state.is_scancode_pressed(Scan::S) {
			self.camera.translate(rot.rotate(&Vec3::new(0.0, 0.0, -dp)));
		}
		if state.is_scancode_pressed(Scan::A) {
			self.camera.translate(rot.rotate(&Vec3::new( dp, 0.0, 0.0)));
		}
		if state.is_scancode_pressed(Scan::D) {
			self.camera.translate(rot.rotate(&Vec3::new(-dp, 0.0, 0.0)));
		}
		if state.is_scancode_pressed(Scan::Q) {
			self.camera.translate(rot.rotate(&Vec3::new(0.0,  dp, 0.0)));
		}
		if state.is_scancode_pressed(Scan::E) {
			self.camera.translate(rot.rotate(&Vec3::new(0.0, -dp, 0.0)));
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
		println!("x:{:.4}, y:{:.4}, z:{:.4}, xrot:{:.4}, yrot:{:.4}", self.camera.get_pos().x, self.camera.get_pos().y, self.camera.get_pos().z, self.camera.get_xrot(), self.camera.get_yrot());
	}
}
