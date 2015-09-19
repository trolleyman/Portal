
use {DT, Vec3, Mat4};
use render::{Render, Mesh};

use cg::{self, Rotation3, Zero, Vector, Angle, Deg};

#[derive(Debug, Clone, Copy)]
pub struct Entity {
	pub pos: Vec3,
	pub vel: Vec3,
	pub mesh: Mesh
}

impl Entity {
	pub fn new(pos: Vec3, vel: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, vel:vel, mesh:mesh }
	}
	pub fn tick(&mut self, dt: DT) {
		self.pos = self.pos + self.vel.mul_s(dt);
	}
	pub fn render(&self, ren: &mut Render) {
		let model_mat: Mat4 = Mat4::from_translation(&self.pos);
		self.mesh.render(ren, &model_mat);
	}
}

#[derive(Clone, Copy)]
pub struct Camera {
	pos: Vec3,
	xrot: f32,
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
		self.xrot += x;
		self.yrot += y;
		self.update_view();
	}
	
	pub fn update_view(&mut self) {
		self.view = Mat4::from(cg::Decomposed {
			scale: 1.0,
			rot: cg::Basis3::from_euler(cg::rad(self.yrot), cg::rad(self.xrot), cg::rad(0.0)),
			disp: self.pos,
		});
	}
}
