use prelude::*;

use render::{Render, Mesh};
use nc::ray::{Ray, RayIntersection};
use na;

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
	w: f32,
	h: f32,
}
impl Portal {
	pub fn new(pos: Vec3, in_normal: Vec3, w: f32, h: f32) -> Portal {
		let normal = in_normal.normalize();
		let tangent = if normal == Vec3::new(0.0, 1.0, 0.0) {
			Vec3::new(0.0, 0.0, 1.0)
		} else {
			Vec3::new(0.0, 1.0, 0.0)
		};
		
		let d = 0.04;
		let w2 = w/2.0;
		let h2 = h/2.0;
		let d2 = d/2.0;
		
		let mesh = Mesh::indexed(
			// 01  45
			// 23  67
			&[
				Pnt3::new(-w2,  h2,  d2), // 0 - Back
				Pnt3::new( w2,  h2,  d2), // 1
				Pnt3::new(-w2, -h2,  d2), // 2
				Pnt3::new( w2, -h2,  d2), // 3
				
				Pnt3::new(-w2,  h2, -d2), // 4 - Front
				Pnt3::new( w2,  h2, -d2), // 5
				Pnt3::new(-w2, -h2, -d2), // 6
				Pnt3::new( w2, -h2, -d2), // 7
			], &[
				na::Vec3::new(0, 2, 3),
				na::Vec3::new(0, 3, 1),
				
				na::Vec3::new(4, 7, 6),
				na::Vec3::new(4, 5, 7),
			], &[
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
				
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
				Vec3::new(1.0, 1.0, 1.0),
			]);
		
		//let mesh = Mesh::new_rectangle_double(w, h, Vec3::new(1.0, 1.0, 1.0));
		let outline_mesh = Mesh::new_rect_torus(w, h, 0.07);
		
		let model_mat = translation_mat(&pos) * get_rotation_between(Vec3::new(0.0, 0.0, -1.0), normal);
		
		Portal {
			pos: pos,
			normal: normal,
			tangent: tangent,
			mesh: mesh,
			outline_mesh: outline_mesh,
			model_mat: model_mat,
			w: w,
			h: h,
		}
	}
	
	pub fn get_model_mat(&self) -> Mat4 {
		self.model_mat
	}
	
	pub fn render(&self, r: &mut Render) {
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		self.mesh.render(r, translation_mat(&self.pos) * rotation);
	}
	pub fn render_color(&self, r: &mut Render, color: &[f32; 4]) {
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		self.mesh.render_color(r, translation_mat(&self.pos) * rotation, color);
	}
	
	pub fn render_outline(&self, r: &mut Render) {
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		self.outline_mesh.render(r, translation_mat(&self.pos) * rotation);
	}
	
	pub fn get_intersection(&self, ray: &Ray<Pnt3>) -> Option<RayIntersection<Vec3>> {
		// a--b
		// |  |
		// c--d
		let (w2, h2) = (Vec3::new(self.w / 2.0, 0.0, 0.0), Vec3::new(0.0, self.h / 2.0, 0.0));
		let rotation = get_rotation_between(Vec3::new(0.0, 0.0, -1.0), self.normal);
		let trans = self.pos;
		let a = <Vec3 as FromHomogeneous<Vec4>>::from(&(rotation * (- w2 - h2).to_homogeneous())).to_pnt()+ trans;
		let b = <Vec3 as FromHomogeneous<Vec4>>::from(&(rotation * (  w2 - h2).to_homogeneous())).to_pnt()+ trans;
		let c = <Vec3 as FromHomogeneous<Vec4>>::from(&(rotation * (- w2 + h2).to_homogeneous())).to_pnt()+ trans;
		let d = <Vec3 as FromHomogeneous<Vec4>>::from(&(rotation * (  w2 + h2).to_homogeneous())).to_pnt()+ trans;
		match ::nc::ray::triangle_ray_intersection(&a, &d, &b, ray) {
			Some((i, _)) => Some(i),
			None => match ::nc::ray::triangle_ray_intersection(&a, &c, &d, ray) {
				Some((i, _)) => Some(i),
				None => None,
			},
		}
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
	
	pub fn transform_through_portal(&mut self, mut p_in: Portal, mut p_out: Portal) {
		let mut cam_n = Vec3::new(0.0, 0.0, 1.0);
		cam_n = Rot3::new_with_euler_angles(0.0, self.xrot, 0.0).rotate(&cam_n);
		cam_n = Rot3::new_with_euler_angles(-self.yrot, 0.0, 0.0).rotate(&cam_n);
		
		if p_in.normal.dot(&cam_n) > 0.0 {
			// Invert
			p_in.normal = -p_in.normal;
			p_out.normal = -p_out.normal;
		}
		
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
	
	// Returns true if the camera has been transformed through a portal.
	fn translate_through_portal(&mut self, mov: Vec3, p_in: &Portal, p_out: &Portal) -> bool {
		let mov_norm = mov.normalize();
		let mov_len = mov.norm();
		if let Some(ri) = p_in.get_intersection(&Ray::new(self.pos.to_pnt(), mov_norm)) {
			if ri.toi <= mov_len {
				// Intersection - translate intersection point to other portal + rotate
				let rot = get_rotation_between(p_in.normal, p_out.normal);
				let init_pos = self.pos;
				self.pos = self.pos + mov;
				self.pos = self.pos - p_in.pos;
				self.pos = FromHomogeneous::from(&(self.pos.to_homogeneous() * rot));
				self.pos = self.pos + p_out.pos;
				
				println!("###### Portal Teleportation ######");
				println!("from: {:?}", init_pos);
				println!("  to: {:?}", self.pos);
				
				// Get x-rotation between p_in.normal and p_out.normal
				let (in_norm2d, out_norm2d) = (Vec2::new(p_in.normal.x, p_in.normal.z), Vec2::new(p_out.normal.x, p_out.normal.z));
				let angle_between = out_norm2d.y.atan2(out_norm2d.x) - in_norm2d.y.atan2(in_norm2d.x);
				println!("angle_between: {}", angle_between);
				self.xrot -= angle_between;
				
				return true;
			}
		}
		false
	}
	
	pub fn translate(&mut self, mov: Vec3, ps: &Option<(Portal, Portal)>) {
		println!("translate POS x:{: >7.4}, y:{: >7.4}, z:{: >7.4} --- MOV x:{: >7.4}, y:{: >7.4}, z:{: >7.4}"
					, self.pos.x, self.pos.y, self.pos.z, mov.x, mov.y, mov.z);
		
		if let &Some((ref p1, ref p2)) = ps {
			if !(self.translate_through_portal(mov, p1, p2) || self.translate_through_portal(mov, p2, p1)) {
				self.pos = self.pos + mov;
			}
		} else {
			self.pos = self.pos + mov;
		}
		
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
