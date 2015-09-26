use prelude::*;

use render::{Render, Mesh};

use nc;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
	Dynamic,
	Static,
}

#[derive(Clone)]
pub struct Entity {
	pos: Vec3,
	vel: Vec3,
	mesh: Mesh,
	etype: EntityType,
}

#[derive(Copy, Clone)]
pub struct Portal {
	pos: Vec3,
	normal: Vec3,
}

impl Entity {
	pub fn new(pos: Vec3, vel: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, vel:vel, mesh:mesh, etype: EntityType::Dynamic }
	}
	pub fn new_static(pos: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, vel: Vec3::new(0.0, 0.0, 0.0), mesh:mesh, etype: EntityType::Static }
	}
	pub fn tick(&mut self, dt: DT) {
		match self.etype {
			EntityType::Dynamic => {
				self.pos = self.pos + self.vel * dt;
			},
			EntityType::Static => {},
		}
	}
	pub fn render(&self, ren: &mut Render) {
		let model_mat: Mat4 = translation_mat(&self.pos);
		self.mesh.render(ren, model_mat);
	}
}

#[derive(Clone, Copy)]
pub struct Camera {
	pos: Vec3,
	xrot: f32,
	yrot: f32,
	fov: f32,
	view: Mat4,
}

impl Camera {
	pub fn new(pos: Vec3, fov: f32) -> Camera {
		let mut cam = Camera {
			pos: pos,
			xrot: 0.0,
			yrot: 0.0,
			fov: fov * (::std::f32::consts::PI / 180.0),
			view: Mat4::new_identity(4),
		};
		cam.update_view();
		cam
	}
	
	pub fn rotate(&mut self, x: f32, y: f32) {
		self.xrot += x / 10.0;
		self.yrot += y / 10.0;
		
		let limit = 1.2;
		if self.yrot < -limit {
			self.yrot = -limit;
		} else if self.yrot > limit {
			self.yrot =  limit;
		}
		let pi2 = 2.0 * ::std::f32::consts::PI;
		self.xrot = self.xrot % pi2;
		if self.xrot < 0.0 {
			self.xrot = self.xrot + pi2;
		}
		
		
		self.update_view();
	}
	
	fn update_view(&mut self) {
		self.view = Rot3::new(Vec3::new(-self.yrot, 0.0, 0.0)).to_homogeneous();
		self.view = self.view * Rot3::new(Vec3::new(0.0, self.xrot, 0.0)).to_homogeneous();
		self.view = self.view * translation_mat(&-self.pos);
	}
	
	pub fn translate(&mut self, mov: Vec3) {
		self.pos = self.pos + mov;
		self.update_view()
	}
	
	pub fn get_view(&self) -> Mat4 {
		self.view
	}
	
	pub fn get_fov(&self) -> f32 {
		self.fov
	}
	
	pub fn get_xrot(&self) -> f32 {
		self.xrot
	}
	
	pub fn get_yrot(&self) -> f32 {
		self.yrot
	}
	
	pub fn get_pos(&self) -> Vec3 {
		self.pos
	}
}
