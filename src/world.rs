use prelude::*;

use entity::{Entity, Camera, Portal};
use sdl2::keyboard::{KeyboardState, Keycode, Mod};
use render::Render;

use gl;

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
		self.render_from_camera(ren, &self.camera);
		
		match self.portals {
			Some((p1, p2)) => {
				let mut p1_transformed_cam = self.camera.clone();
				p1_transformed_cam.transform_through_portal(&p1, &p2);
				let mut p2_transformed_cam = self.camera.clone();
				p2_transformed_cam.transform_through_portal(&p2, &p1);
				
				unsafe {
					// Write stencil
					gl::Enable(gl::STENCIL_TEST);
					gl::StencilMask(0xFF);
					gl::ClearStencil(0);
					gl::Clear(gl::STENCIL_BUFFER_BIT);
					
					gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
					gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
					gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
					gl::DepthMask(gl::FALSE);
					
					p1.render(ren);
					
					gl::StencilFunc(gl::ALWAYS, 2, 0xFF);
					p2.render(ren);
					
					// Read stencil
					gl::StencilMask(0x00);
					gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
					gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
					gl::DepthMask(gl::TRUE);
					
					gl::StencilFunc(gl::GREATER, 0x0, 0xFF);
					p1.render(ren);
					p2.render(ren);
					gl::Clear(gl::DEPTH_BUFFER_BIT);
					
					gl::StencilFunc(gl::EQUAL, 0x1, 0xFF);
					self.render_from_camera(ren, &p1_transformed_cam);
					
					gl::StencilFunc(gl::EQUAL, 0x2, 0xFF);
					self.render_from_camera(ren, &p2_transformed_cam);
					
					gl::Disable(gl::STENCIL_TEST);
				}
			},
			None => {},
		}
		
	}
	fn render_from_camera(&self, ren: &mut Render, cam: &Camera) {
		ren.set_camera(cam);
		for ent in self.entities.iter() {
			ent.render(ren);
		}
		match self.portals {
			Some((p1, p2)) => {
				p1.render_outline(ren);
				p2.render_outline(ren);
			},
			None => {}
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
