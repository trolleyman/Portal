use prelude::*;

use render::{Render, Mesh};

use cg::{self, Angle, Deg};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
	Dynamic,
	Static,
}

#[derive(Debug, Copy, Clone)]
pub struct Entity {
	pos: Vec3,
	vel: Vec3,
	mesh: Mesh,
	etype: EntityType,
}

impl Entity {
	pub fn new(pos: Vec3, vel: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, vel:vel, mesh:mesh, etype: EntityType::Dynamic }
	}
	pub fn new_static(pos: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, vel: Vec3::zero(), mesh:mesh, etype: EntityType::Static }
	}
	pub fn tick(&mut self, dt: DT) {
		match self.etype {
			EntityType::Dynamic => {
				self.pos = self.pos + self.vel.mul_s(dt);
			},
			EntityType::Static => {},
		}
	}
	pub fn render(&self, ren: &mut Render) {
		let model_mat: Mat4 = Mat4::from_translation(&self.pos);
		self.mesh.render(ren, model_mat);
	}
}

#[derive(Clone, Copy)]
pub struct Camera {
	pub pos: Vec3,
	pub xrot: f32,
	yrot: f32,
	fov: Deg<f32>,
	view: Mat4,
}

impl Camera {
	pub fn new(pos: Vec3, fov: f32) -> Camera {
		let mut cam = Camera {
			pos: pos,
			xrot: 0.0,
			yrot: 0.0,
			fov: Deg{s: fov},
			view: Mat4::zero(),
		};
		cam.update_view();
		cam
	}
	
	pub fn rotate(&mut self, x: f32, y: f32) {
		self.xrot += x / 10.0;
		self.yrot += y / 10.0;
		self.update_view();
	}
	
	fn update_view(&mut self) {
		self.view = Mat4::from(cg::Decomposed {
			scale: 1.0,
			rot: cg::Basis3::from_euler(cg::rad(self.yrot), cg::rad(0.0), cg::rad(0.0)),
			disp: Vec3::zero(),
		});
		self.view = self.view * Mat4::from(cg::Decomposed {
			scale: 1.0,
			rot: cg::Basis3::from_euler(cg::rad(0.0), cg::rad(self.xrot), cg::rad(0.0)),
			disp: Vec3::zero(),
		});
		
		self.view = self.view * Mat4::from_translation(&-self.pos);
	}
	
	pub fn translate(&mut self, mov: Vec3) {
		self.pos = self.pos + mov;
		self.update_view()
	}
	
	pub fn get_view(&self) -> Mat4 {
		self.view
	}
	
	pub fn get_fov(&self) -> Deg<f32> {
		self.fov
	}
}
