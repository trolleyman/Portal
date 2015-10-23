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
		            else if state.is_scancode_pressed(Scan::LCtrl) || state.is_scancode_pressed(Scan::RCtrl) {0.1}
		            else {0.5};
		let dp = speed * dt;
		let rot = Rot3::new(Vec3::new(0.0, -self.camera.get_xrot(), 0.0));
		let mut mov = Vec3::new(0.0, 0.0, 0.0);
		if state.is_scancode_pressed(Scan::W) {
			mov = mov + rot.rotate(&Vec3::new(0.0, 0.0,  dp));
		}
		if state.is_scancode_pressed(Scan::S) {
			mov = mov + rot.rotate(&Vec3::new(0.0, 0.0, -dp));
		}
		if state.is_scancode_pressed(Scan::A) {
			mov = mov + rot.rotate(&Vec3::new( dp, 0.0, 0.0));
		}
		if state.is_scancode_pressed(Scan::D) {
			mov = mov + rot.rotate(&Vec3::new(-dp, 0.0, 0.0));
		}
		if state.is_scancode_pressed(Scan::Q) {
			mov = mov + rot.rotate(&Vec3::new(0.0,  dp, 0.0));
		}
		if state.is_scancode_pressed(Scan::E) {
			mov = mov + rot.rotate(&Vec3::new(0.0, -dp, 0.0));
		}
		if mov != Vec3::new(0.0, 0.0, 0.0) {
			self.camera.translate(mov, &self.portals.clone());
		}
	}
	
	pub fn render(&self, ren: &mut Render) {
		ren.set_camera(&self.camera);
		
		match self.portals {
			Some((p1, p2)) => {
				if ren.should_render_portals() {
					let mut p1_transformed_cam = self.camera.clone();
					p1_transformed_cam.transform_through_portal(p1, p2);
					let mut p2_transformed_cam = self.camera.clone();
					p2_transformed_cam.transform_through_portal(p2, p1);
					
					unsafe {
						gl::Enable(gl::STENCIL_TEST);
						
						// 2. Draw portal 1 in the depth buffer
						gl::StencilMask(0x00);
						gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
						gl::DepthMask(gl::TRUE);
						p1.render(ren);
						// 3. Draw portal 2 in the stencil buffer with 2s
						gl::StencilMask(0xFF);
						gl::StencilFunc(gl::ALWAYS, 2, 0xFF);
						gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
						gl::DepthMask(gl::FALSE);
						p2.render(ren);
						
						// 4. Clear the depth buffer
						gl::StencilMask(0x00);
						gl::DepthMask(gl::TRUE);
						gl::Clear(gl::DEPTH_BUFFER_BIT);
						
						// 5. Draw portal 2 in the depth buffer
						gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
						gl::DepthMask(gl::TRUE);
						p2.render(ren);
						// 6. Draw portal 1 in the stencil buffer with 1s
						gl::StencilMask(0xFF);
						gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
						gl::DepthMask(gl::FALSE);
						p1.render(ren);
						
						// 7. Clear the Depth Buffer
						gl::StencilMask(0x00);
						gl::DepthMask(gl::TRUE);
						gl::Clear(gl::DEPTH_BUFFER_BIT);
						
						//  8. Draw scene through portal 1 in the 1s
						gl::StencilFunc(gl::EQUAL, 0x1, 0xFF);
						gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
						gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
						gl::DepthMask(gl::TRUE);
						self.render_from_camera(ren, &p1_transformed_cam);
						
						//  9 Draw scene through portal 2 in the 2s
						gl::StencilFunc(gl::EQUAL, 0x2, 0xFF);
						self.render_from_camera(ren, &p2_transformed_cam);
						
						// 10. Draw portal 1 in the depth buffer to protect portal 1
						gl::StencilFunc(gl::ALWAYS, 0x00, 0xFF);
						gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
						p1.render(ren);
						
						// 11. Draw portal 2 in the depth buffer to protect portal 2
						p2.render(ren);
						
						// 12. Draw main scene
						gl::Disable(gl::STENCIL_TEST);
						gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
					}
				}
				if ren.is_wireframe() && !ren.should_render_portals() {
					p1.render(ren);
					p2.render(ren);
				}
			},
			_ => {},
		}
		
		self.render_from_camera(ren, &self.camera);
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
		
		ren.set_camera(&self.camera);
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
		print!("x:{:.4}, y:{:.4}, z:{:.4}, xrot:{:.4}, yrot:{:.4}", self.camera.get_pos().x, self.camera.get_pos().y, self.camera.get_pos().z, self.camera.get_xrot(), self.camera.get_yrot());
	}
}
