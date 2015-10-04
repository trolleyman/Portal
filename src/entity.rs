use prelude::*;

use render::{Render, Mesh};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
	Dynamic,
	Static,
}

#[derive(Copy, Clone)]
pub struct Entity {
	pos: Vec3,
	vel: Vec3,
	mesh: Mesh,
	etype: EntityType,
}

#[derive(Copy, Clone)]
pub struct Portal {
	pub pos: Vec3,
	pub normal: Vec3,
	tangent: Vec3,
	mesh: Mesh,
	outline_mesh: Mesh,
	model_mat: Mat4,
}
impl Portal {
	pub fn new(pos: Vec3, in_normal: Vec3, w: f32, h: f32) -> Portal {
		let normal = in_normal.normalize();
		let tangent = if normal == Vec3::new(0.0, 1.0, 0.0) {
			Vec3::new(0.0, 0.0, 1.0)
		} else {
			Vec3::new(0.0, 1.0, 0.0)
		};
		let mesh = Mesh::new_rectangle(w, h, Vec3::new(1.0, 1.0, 1.0));
		let outline_mesh = Mesh::new_rect_torus(w, h, 0.07);
		
		let model_mat = translation_mat(&pos) * get_rotation_between(Vec3::new(0.0, 0.0, -1.0), normal);
		
		Portal {
			pos: pos,
			normal: normal,
			tangent: tangent,
			mesh: mesh,
			outline_mesh: outline_mesh,
			model_mat: model_mat,
		}
	}
	
	pub fn get_model_mat(&self) -> Mat4 {
		self.model_mat
	}
	
	pub fn render(&self, r: &mut Render) {
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		self.mesh.render(r, translation_mat(&self.pos) * rotation);
	}
	
	pub fn render_outline(&self, r: &mut Render) {
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		self.outline_mesh.render(r, translation_mat(&self.pos) * rotation);
	}
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

#[derive(Copy, Clone)]
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
	
	pub fn transform_through_portal(&mut self, p_in: &Portal, p_out: &Portal) {
		self.pos = self.pos - p_in.pos;
		let rot = get_rotation_between(p_in.normal, p_out.normal);
		self.pos = FromHomogeneous::from(&(self.pos.to_homogeneous() * rot));
		self.pos = self.pos + p_out.pos;
		
		self.view = Rot3::new(Vec3::new(-self.yrot, 0.0, 0.0)).to_homogeneous();
		self.view = self.view * Rot3::new(Vec3::new(0.0, self.xrot, 0.0)).to_homogeneous();
		self.view = self.view * rot;
		/*self.view = self.view * get_rotation_between(p_in.normal, -p_out.normal);
		match rot.inv() {
			Some(m) => self.view = self.view * m,
			None => {},
		}*/
		self.view = self.view * translation_mat(&-self.pos);	
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
