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
	pub pos: Vec3,
	pub vel: Vec3,
	rot: Rot3,
	mesh: Mesh,
	etype: EntityType,
}

#[derive(Copy, Clone)]
pub struct Portal {
	pub pos: Vec3,
	pub rot: Rot3,
	mesh: Mesh,
	outline_mesh: Mesh,
	w: f32,
	h: f32,
}
impl Portal {
	pub fn new(pos: Vec3, rot: Rot3, w: f32, h: f32) -> Portal {
		let d = 0.00;
		let w2 = w/2.0;
		let h2 = h/2.0;
		let d2 = d/2.0;
		
		let mesh = Mesh::indexed(
			// 01  45
			// 23  67
			&[
				Vec3::new(-w2,  h2,  d2), // 0 - Back
				Vec3::new( w2,  h2,  d2), // 1
				Vec3::new(-w2, -h2,  d2), // 2
				Vec3::new( w2, -h2,  d2), // 3
				
				Vec3::new(-w2,  h2, -d2), // 4 - Front
				Vec3::new( w2,  h2, -d2), // 5
				Vec3::new(-w2, -h2, -d2), // 6
				Vec3::new( w2, -h2, -d2), // 7
			], &[
				na::Vec3::new(0, 3, 2),
				na::Vec3::new(0, 1, 3),
				
				na::Vec3::new(4, 6, 7),
				na::Vec3::new(4, 7, 5),
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
		let outline_mesh = Mesh::new_rect_torus(w, h, 0.04);
				
		Portal {
			pos: pos,
			rot: rot,
			mesh: mesh,
			outline_mesh: outline_mesh,
			w: w,
			h: h,
		}
	}
	
	pub fn get_normal(&self) -> Vec3 {
		self.rot * Vec3::new(0.0, 0.0, 1.0)
	}
	
	pub fn get_model_mat(&self) -> Mat4 {
		Iso3::new_with_rotmat(self.pos, self.rot).to_homogeneous()
	}
	
	pub fn render(&self, r: &mut Render) {
		self.mesh.render(r, self.get_model_mat());
	}
	pub fn render_color(&self, r: &mut Render, color: &[f32; 4]) {
		self.mesh.render_color(r, self.get_model_mat(), color);
	}
	
	pub fn render_outline(&self, r: &mut Render) {
		self.outline_mesh.render(r, self.get_model_mat());
	}
	
	pub fn get_intersection(&self, ray: &Ray<Pnt3>) -> Option<RayIntersection<Vec3>> {
		// a--b
		// |  |
		// c--d
		let (w2, h2) = (Vec3::new(self.w / 2.0, 0.0, 0.0), Vec3::new(0.0, self.h / 2.0, 0.0));
		
		let a = (self.rot * (- w2 - h2)).to_pnt() + self.pos;
		let b = (self.rot * (  w2 - h2)).to_pnt() + self.pos;
		let c = (self.rot * (- w2 + h2)).to_pnt() + self.pos;
		let d = (self.rot * (  w2 + h2)).to_pnt() + self.pos;
		
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
		Entity{ pos:pos, rot: Rot3::new_identity(3), vel:vel, mesh:mesh, etype: EntityType::Dynamic }
	}
	pub fn new_static(pos: Vec3, mesh: Mesh) -> Entity {
		Entity{ pos:pos, rot: Rot3::new_identity(3), vel: Vec3::new(0.0, 0.0, 0.0), mesh:mesh, etype: EntityType::Static }
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

// /// Gets the matrix that transforms the world from p_in to p_out
// pub fn get_portal_transform(p_in: &Portal, p_out: &Portal) -> Mat4 {
// 	let rot = get_rotation_between(p_in.normal, p_out.normal);
	
	
// }

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
		let mut rot = p_in.rot.rotation_to(&p_out.rot);
		
		//let rot = ::get_rotation_between(p_in_norm, p_out_norm);
		self.pos = self.pos - p_in.pos;
		self.pos = rot * self.pos;
		self.pos = self.pos + p_out.pos;
		
		self.view = (Rot3::new_with_euler_angles(-self.yrot, 0.0, 0.0) * Rot3::new_with_euler_angles(0.0, self.xrot, 0.0)).to_homogeneous();
		self.view = self.view * rot.inv().unwrap_or(Rot3::new_identity(3)).to_homogeneous();
		self.view = self.view * translation_mat(&-self.pos);
		
		/*self.view = self.view * get_rotation_between(p_in_norm, -p_out_norm);
		match rot.inv() {
			Some(m) => self.view = self.view * m,
			None => {},
		}*/
	}
	// // Transforms the current camera through a portal n number of times.
	// pub fn transform_through_portals(&mut self, p_in: Portal, p_out: Portal, n: u32) {
	// 	for _ in 0..n {
			
	// 	}
	// }
	
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
		self.view = (Rot3::new_with_euler_angles(-self.yrot, 0.0, 0.0) * Rot3::new_with_euler_angles(0.0, self.xrot, 0.0)).to_homogeneous();
		self.view = self.view * translation_mat(&-self.pos);
	}
	
	// Returns true if the camera has been translated through a portal.
	fn translate_through_portal(&mut self, mov: Vec3, p_in: &Portal, p_out: &Portal) -> bool {
		let mov_norm = mov.normalize();
		let mov_len = mov.norm();
		if let Some(ri) = p_in.get_intersection(&Ray::new(self.pos.to_pnt(), mov_norm)) {
			if ri.toi <= mov_len {
				// Intersection - translate intersection point to other portal + rotate
				let rot = p_in.rot.rotation_to(&p_out.rot);
				let init_pos = self.pos;
				self.pos = self.pos + mov;
				self.pos = self.pos - p_in.pos;
				self.pos = rot * self.pos;
				self.pos = self.pos + p_out.pos;
				
				println!("###### Portal Teleportation ######");
				println!("from: {:?}", init_pos);
				println!("  to: {:?}", self.pos);
				
				// Get x-rotation between p_in.normal and p_out.normal
				let p_in_norm  = p_in .get_normal();
				let p_out_norm = p_out.get_normal();
				let (in_norm2d, out_norm2d) = (Vec2::new(p_in_norm.x, p_in_norm.z), Vec2::new(p_out_norm.x, p_out_norm.z));
				let angle_between_x = out_norm2d.y.atan2(out_norm2d.x) - in_norm2d.y.atan2(in_norm2d.x);
				let angle_between_y = 0.0; // out_norm2d.x.atan2(out_norm2d.y) - in_norm2d.x.atan2(in_norm2d.y);
				println!("angle_between_x: {}", angle_between_x);
				println!("angle_between_y: {}", angle_between_y);
				self.xrot += angle_between_x;
				self.yrot += angle_between_y;
				
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
