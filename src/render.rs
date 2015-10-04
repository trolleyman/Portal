use prelude::*;

use entity::Camera;

use std::fs::File;
use std::io::prelude::*;
use std::ops::Drop;
use std::mem;
use std::ptr::null;
use std::ffi::CString;

use na;
//use rand::{Rand, XorShiftRng, SeedableRng, Rng};

use sdl2::video::{GLContext, Window};

use gl::{self};
use gl::types::*;

pub type Index = GLushort;

pub struct Render<'a> {
	pub win: &'a mut Window,
	pub gl_context: &'a mut GLContext,
	pub main_shader: Shader,
	pub vp_mat: Mat4,
	pub m_mat: Mat4,
	// arrow_mesh: Mesh,
	view_wireframes: bool,
}

impl<'a> Render<'a> {
	pub fn new(win: &'a mut Window, context: &'a mut GLContext) -> Render<'a> {
		//let _ = win.gl_set_context_to_current();
		let ren = Render {
			win: win,
			gl_context: context,
			main_shader: match Shader::from_files("shaders/main.vs", "shaders/main.fs") {
					Ok(s) => s,
					Err(e) => panic!("{}", e),
				},
			vp_mat: Mat4::new_identity(4),
			m_mat: Mat4::new_identity(4),
			view_wireframes: false,
		};
		unsafe {
			// gl::Enable(gl::CULL_FACE);
			gl::Enable(gl::DEPTH_TEST);
			// gl::Enable(gl::LINE_SMOOTH);
			// gl::LineWidth(1.0);
			gl::DepthFunc(gl::LESS);
		}
		ren.win.subsystem().gl_set_swap_interval(1);
		
		ren
	}
	
	pub fn swap(&mut self) {
		self.win.show();
		self.win.gl_swap_window();
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.3, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::Clear(gl::DEPTH_BUFFER_BIT);
			
			if self.view_wireframes {
				gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
				gl::Disable(gl::DEPTH_TEST);
				gl::Disable(gl::CULL_FACE);
			} else {
				gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
				gl::Enable(gl::DEPTH_TEST);
				gl::Enable(gl::CULL_FACE);
			}
		}
	}
	
	pub fn set_camera(&mut self, cam: &Camera) {
		// Recalculate VP matrix
		let (w, h) = self.win.drawable_size();
		let projection = Persp3::new(w as f32 / h as f32, cam.get_fov(), 0.01, 500.0).to_mat();
		let view = cam.get_view();
		self.vp_mat = projection * view;
	}
	
	pub fn set_model_mat(&mut self, mat: Mat4) {
		self.m_mat = mat;
		self.main_shader.set_mvp(self.vp_mat * self.m_mat);
	}
	
	pub fn get_drawable_size(&self) -> (u32, u32) {
		self.win.drawable_size()
	}
	pub fn update_size(&mut self) {
		let (w, h) = self.win.drawable_size();
		unsafe {
			gl::Viewport(0, 0, w as i32, h as i32);
		}
	}
	pub fn toggle_wireframes(&mut self) {
		self.view_wireframes = !self.view_wireframes;
	}
	
	// pub fn render_arrow(pos: Vec3, vec: Vec3) {
	// 	arrow_mesh.render()
	// }
}

pub struct Shader {
	prog: GLuint,
	vs: GLuint,
	fs: GLuint,
	mvp_pos: GLint,
}

impl Shader {
	pub fn from_files(vs: &str, fs: &str) -> Result<Shader, String> {
		let mut vs_file = match File::open(vs) {
			Ok(f) => f,
			Err(e) => return Err(format!("error opening file '{}': {}", vs, e)),
		};
		let mut vs_str = Vec::new();
		match vs_file.read_to_end(&mut vs_str) {
			Ok(_) => {},
			Err(e) => return Err(format!("error reading file '{}': {}", vs, e)),
		};
		
		let mut fs_file = match File::open(fs) {
			Ok(f) => f,
			Err(e) => return Err(format!("error opening file '{}': {}", fs, e)),
		};
		let mut fs_str = Vec::new();
		match fs_file.read_to_end(&mut fs_str) {
			Ok(_) => {},
			Err(e) => return Err(format!("error reading file '{}': {}", fs, e)),
		};
		
		Shader::from_strs(&vs_str, &fs_str)
	}
	pub fn from_strs(vs_str: &[u8], fs_str: &[u8]) -> Result<Shader, String> {
		unsafe {
			let prog = gl::CreateProgram();
			let vs = gl::CreateShader(gl::VERTEX_SHADER);
			let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
			
			try!(Shader::compile_shader(vs, vs_str));
			try!(Shader::compile_shader(fs, fs_str));
			
			gl::AttachShader(prog, vs);
			gl::AttachShader(prog, fs);
			
			gl::BindAttribLocation(prog, 0, "in_pos".as_ptr() as *const i8);
			gl::BindAttribLocation(prog, 1, "in_color".as_ptr() as *const i8);
			
			gl::LinkProgram(prog);
			
			let mut is_linked: GLint = 0;
			gl::GetProgramiv(prog, gl::LINK_STATUS, &mut is_linked);
			if is_linked == gl::FALSE as GLint {
				let mut len: GLint = 0;
				gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
				
				let mut log: Vec<u8> = Vec::with_capacity(len as usize);
				gl::GetProgramInfoLog(prog, len, &mut len, log.as_mut_ptr() as *mut i8);
				log.set_len(len as usize);
				
				let mut s = String::from("error linking program: \n");
				s.push_str(&String::from_utf8_lossy(&log));
				return Err(s);
			}
			
			gl::UseProgram(prog);
						
			Ok(Shader {
				prog: prog,
				vs: vs,
				fs: fs,
				mvp_pos: gl::GetUniformLocation(prog, CString::new("in_mvp").unwrap().as_ptr()),
			})
		}
	}
	
	fn compile_shader(id: GLuint, src: &[u8]) -> Result<(), String> {
		unsafe {
			gl::ShaderSource(id, 1, ::std::mem::transmute(&src.as_ptr()), ::std::mem::transmute(&src.len()));
			gl::CompileShader(id);
			
			let mut is_compiled: GLint = 0;
			gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut is_compiled);
			if is_compiled == gl::FALSE as GLint {
				let mut len: GLint = 0;
				gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
				
				let mut log: Vec<u8> = Vec::with_capacity(len as usize);
				gl::GetShaderInfoLog(id, len, &mut len, log.as_ptr() as *mut i8);
				log.set_len(len as usize);
				
				let mut s = String::from("error compiling shader: \n");
				s.push_str(&String::from_utf8_lossy(&log));
				return Err(s);
			}
		}
		
		Ok(())
	}
	
	pub fn use_prog(&self) {
		unsafe { gl::UseProgram(self.prog) }
	}
	
	pub fn set_mvp(&self, mvp: Mat4) {
		unsafe {
			gl::UniformMatrix4fv(self.mvp_pos, 1, gl::FALSE, &mvp.as_array()[0][0] as *const GLfloat);
		}
	}
}
impl Drop for Shader {
	fn drop(&mut self) {
		unsafe {
			gl::DetachShader(self.prog, self.vs);
			gl::DetachShader(self.prog, self.fs);
			gl::DeleteProgram(self.prog);
			gl::DeleteShader(self.vs);
			gl::DeleteShader(self.fs);
		}
	}
}

pub struct MeshBuilder {
	verts: Vec<Pnt3>,
	colors: Vec<Vec3>,
	indices: Vec<na::Vec3<Index>>,
}
impl MeshBuilder {
	pub fn new() -> MeshBuilder {
		MeshBuilder {
			verts: Vec::new(),
			colors: Vec::new(),
			indices: Vec::new(),
		}
	}
	
	pub fn push(&mut self, vert: Pnt3, color: Vec3) -> Index {
		self.verts.push(vert);
		self.colors.push(color);
		self.verts.len() as Index - 1
	}
	pub fn index(&mut self, i: na::Vec3<Index>) {
		self.indices.push(i);
	}
	
	pub fn finish(&self) -> Mesh {
		if self.indices.len() == 0 {
			// println!("===============================");
			// println!("verts:   {:?}", self.verts);
			// println!("colors:  {:?}", self.colors);
			Mesh::new(&self.verts, &self.colors)
		} else {
			Mesh::indexed(&self.verts, &self.indices, &self.colors)
		}
	}
}

// This makes it so that Meshes leak during the program as entities die, but it does speed up the process of cloning the world
#[derive(Debug, Copy, Clone)]
pub struct Mesh {
	vao: GLuint,
	vert_len: GLsizei,
	len: GLsizei,
	indices: Option<GLuint>,
	verts: GLuint,
	colors: GLuint,
}
impl Mesh {
	pub fn indexed(verts: &[Pnt3], indices: &[na::Vec3<Index>], colors: &[Vec3]) -> Mesh {
		unsafe {
			// println!("===============================");
			// println!("verts:   {:?}", verts);
			// println!("colors:  {:?}", colors);
			// println!("indices: {:?}", indices);
			
			let mut m = Mesh::new(verts, colors);
			
			let mut inds = 0;
			gl::GenBuffers(1, &mut inds);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, inds);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * mem::size_of::<na::Vec3<Index>>()) as i64, mem::transmute(indices.as_ptr()), gl::STATIC_DRAW);
			gl::Flush();
			
			m.indices = Some(inds);
			m.len = indices.len() as GLsizei * 3;
			m
		}
	}
	
	pub fn new(verts: &[Pnt3], colors: &[Vec3]) -> Mesh {
		unsafe {
			let mut vao: GLuint = 0;
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			
			let mut vbo: [GLuint; 2] = [0, 0];
			gl::GenBuffers(2, &mut vbo[0]);
			
			// Specify that the data to be pushed is the verts
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
			// Push the verts to the GPU
			gl::BufferData(gl::ARRAY_BUFFER, (verts.len() * mem::size_of::<Vec3>()) as i64, mem::transmute(verts.as_ptr()), gl::STATIC_DRAW);
			// Specify that it is attribute 0
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
			gl::EnableVertexAttribArray(0);
			
			// Do the same, but with the colors
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
			gl::BufferData(gl::ARRAY_BUFFER, (colors.len() * mem::size_of::<Vec3>()) as i64, mem::transmute(colors.as_ptr()), gl::STATIC_DRAW);
			gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());
			gl::EnableVertexAttribArray(1);
			
			// So that we copy the verts + colors over before they are freed...
			gl::Flush();
			
			Mesh {
				vao: vao,
				vert_len: verts.len() as GLsizei,
				len: verts.len() as GLsizei,
				indices: None,
				verts: vbo[0],
				colors: vbo[1],
			}
		}
	}
	
	pub fn new_rect_torus(w: f32, h: f32, d: f32) -> Mesh {
		/*let plane_x = normal.cross(&tangent).normalize();
		let x = plane_x * w;
		let out_x = x + plane_x * d;
		let plane_y = normal.cross(&plane_x).normalize();
		let y = plane_y * h;
		let out_y = y + plane_y * d;
		let plane_z = normal.normalize();
		let z = plane_z * d;*/
		let plane_x = Vec3::new(1.0, 0.0, 0.0);
		let x = plane_x * (w / 2.0);
		let out_x = x + plane_x * d;
		let plane_y = Vec3::new(0.0, 1.0, 0.0);
		let y = plane_y * (h / 2.0);
		let out_y = y + plane_y * d;
		let plane_z = Vec3::new(0.0, 0.0, -1.0);
		let z = plane_z * d;
		
		// -Z plane
		// 0  1----4  5
		// 2  3    6  7
		// |          |
		// |          |
		// 8  9   12 13
		// 10 11--14 15
		//
		// 0,5,7,2 - 0,1,11,10 - 4,5,15,14 - 8,13,15,10
		//
		// +Z plane
		// 16 17--20 21
		// 18 19  22 23
		// |          |
		// |          |
		// 24 25  28 29
		// 26 27--30 31
		// 
		// 16,18,23,21 - 16,26,27,17 - 24,26,31,29 - 20,30,31,21
		//
		// Sides   - lurd - 16,0,10,26 - 16,21,5,0 - 21,31,15,5 - 31,26,10,15
		// Insides - lurd - 19,25,9,3 - 22,19,3,6 - 28,22,6,12 - 25,28,12,19
		
		let color: [Vec3; 32] = [Vec3::new(1.0,1.0,1.0); 32];
		/*let mut rng = XorShiftRng::from_seed([666, 14124, 15775644, 194912]);
		for i in 0..32 {
			color[i] = Vec3::new(Rand::rand(&mut rng), Rand::rand(&mut rng), Rand::rand(&mut rng));
		}*/
		
		Mesh::indexed(&[
				*(- out_x + out_y - z).as_pnt(), // 0
				*(-     x + out_y - z).as_pnt(),
				*(- out_x +     y - z).as_pnt(),
				*(-     x +     y - z).as_pnt(),
				
				*(      x + out_y - z).as_pnt(), // 4
				*(  out_x + out_y - z).as_pnt(),
				*(      x +     y - z).as_pnt(),
				*(  out_x +     y - z).as_pnt(),
				
				*(- out_x -     y - z).as_pnt(), // 8
				*(-     x -     y - z).as_pnt(),
				*(- out_x - out_y - z).as_pnt(),
				*(-     x - out_y - z).as_pnt(),
				
				*(      x -     y - z).as_pnt(), // 12
				*(  out_x -     y - z).as_pnt(),
				*(      x - out_y - z).as_pnt(),
				*(  out_x - out_y - z).as_pnt(),
				
				*(- out_x + out_y + z).as_pnt(), // 16
				*(-     x + out_y + z).as_pnt(),
				*(- out_x +     y + z).as_pnt(),
				*(-     x +     y + z).as_pnt(),
				
				*(      x + out_y + z).as_pnt(), // 20
				*(  out_x + out_y + z).as_pnt(),
				*(      x +     y + z).as_pnt(),
				*(  out_x +     y + z).as_pnt(),
				
				*(- out_x -     y + z).as_pnt(), // 24
				*(-     x -     y + z).as_pnt(),
				*(- out_x - out_y + z).as_pnt(),
				*(-     x - out_y + z).as_pnt(),
				
				*(      x -     y + z).as_pnt(), // 28
				*(  out_x -     y + z).as_pnt(),
				*(      x - out_y + z).as_pnt(),
				*(  out_x - out_y + z).as_pnt(),
			], &[
				// Bottom
					na::Vec3::new(0, 7, 5),
					na::Vec3::new(7, 0, 2),
					
					na::Vec3::new(0, 11, 1),
					na::Vec3::new(11, 0, 10),
					
					na::Vec3::new(4, 15, 5),
					na::Vec3::new(15, 4, 14),
					
					na::Vec3::new(8, 15, 13),
					na::Vec3::new(15, 8, 10),
				
				// Top
					na::Vec3::new(16, 23, 18),
					na::Vec3::new(23, 16, 21),
					
					na::Vec3::new(16, 27, 26),
					na::Vec3::new(27, 16, 17),
					
					na::Vec3::new(24, 31, 26),
					na::Vec3::new(31, 24, 29),
					
					na::Vec3::new(20, 31, 30),
					na::Vec3::new(31, 20, 21),
				
				// Sides
					na::Vec3::new(16, 10, 0),
					na::Vec3::new(10, 16, 26),
					
					na::Vec3::new(16, 5, 21),
					na::Vec3::new(5, 16, 0),
					
					na::Vec3::new(21, 15, 31),
					na::Vec3::new(15, 21, 5),
					
					na::Vec3::new(31, 10, 26),
					na::Vec3::new(10, 31, 15),
					
				// Insides
					na::Vec3::new(19, 9, 25),
					na::Vec3::new(9, 19, 3),
					
					na::Vec3::new(22, 3, 19),
					na::Vec3::new(3, 22, 6),
					
					na::Vec3::new(28, 6, 22),
					na::Vec3::new(6, 28, 12),
					
					na::Vec3::new(25, 12, 28),
					na::Vec3::new(12, 25, 9),
				
			], &color)
	}
	pub fn new_rectangle(w: f32, h: f32, color: Vec3) -> Mesh {
		//let plane_x = normal.cross(&tangent).normalize() * w;
		//let plane_y = normal.cross(&plane_x).normalize() * h;
		let plane_x = Vec3::new(w / 2.0, 0.0, 0.0);
		let plane_y = Vec3::new(0.0, h / 2.0, 0.0);
		
		Mesh::indexed(&[
				*(- plane_x + plane_y).as_pnt(),
				*(- plane_x - plane_y).as_pnt(),
				*(  plane_x - plane_y).as_pnt(),
				*(  plane_x + plane_y).as_pnt(),
			], &[
				na::Vec3::new(0, 2, 1),
				na::Vec3::new(0, 3, 2),
			], &[
				color,
				color,
				color,
				color,
			])
	}
	pub fn new_triangle(scale: f32) -> Mesh {
		Mesh::indexed(&[
			Pnt3::new(-0.5,  0.0, 0.0) * scale,
			Pnt3::new( 0.5,  0.0, 0.0) * scale,
			Pnt3::new( 0.0,  1.0, 0.0) * scale,
		], &[
			na::Vec3::new(0, 1, 2),
			na::Vec3::new(0, 2, 1),
		], &[
			Vec3::new(1.0, 0.0, 0.0),
			Vec3::new(0.0, 1.0, 0.0),
			Vec3::new(0.0, 0.0, 1.0),
		])
	}
	pub fn new_square(scale: f32) -> Mesh {
		Mesh::indexed(&[
			Pnt3::new(-0.5,  1.0, 0.0) * scale,
			Pnt3::new(-0.5,  0.0, 0.0) * scale,
			Pnt3::new( 0.5,  0.0, 0.0) * scale,
			Pnt3::new( 0.5,  1.0, 0.0) * scale,
		], &[
			na::Vec3::new(0, 1, 2),
			na::Vec3::new(2, 3, 0),
			
			na::Vec3::new(0, 2, 1),
			na::Vec3::new(2, 0, 3),
		], &[
			Vec3::new(1.0, 0.0, 0.0),
			Vec3::new(0.0, 1.0, 0.0),
			Vec3::new(0.0, 0.0, 1.0),
			Vec3::new(1.0, 1.0, 1.0),
		])
	}
	pub fn new_planes(num_w: u32, num_h: u32, w: f32, h: f32, color1: Vec3, color2: Vec3) -> Mesh {
		let mut mb = MeshBuilder::new();
		let offset_x: f32 = w as f32 / 2.0;
		let offset_y: f32 = h as f32 / 2.0;
		for y in 0..num_h {
			for x in 0..num_w {
				let col = if (x + y) % 2 == 0 {
					color1
				} else {
					color2
				};
				let mut i = [0 as Index, 0, 0, 0];
				i[0] = mb.push(Pnt3::new((x as f32      ) * (w / num_w as f32) - offset_x, 0.0, (y as f32      ) * (h / num_h as f32) - offset_y), col);
				i[1] = mb.push(Pnt3::new((x as f32      ) * (w / num_w as f32) - offset_x, 0.0, (y as f32 + 1.0) * (h / num_h as f32) - offset_y), col);
				i[2] = mb.push(Pnt3::new((x as f32 + 1.0) * (w / num_w as f32) - offset_x, 0.0, (y as f32 + 1.0) * (h / num_h as f32) - offset_y), col);
				i[3] = mb.push(Pnt3::new((x as f32 + 1.0) * (w / num_w as f32) - offset_x, 0.0, (y as f32      ) * (h / num_h as f32) - offset_y), col);
				
				mb.index(na::Vec3::new(i[0], i[1], i[2]));
				mb.index(na::Vec3::new(i[2], i[3], i[0]));
			}
		}
		mb.finish()
	}
	
	pub fn render(&self, ren: &mut Render, model_mat: Mat4) {
		unsafe {
			ren.set_model_mat(model_mat);
			ren.main_shader.use_prog();
			
			gl::BindVertexArray(self.vao);
			
			match self.indices {
				Some(_) => {
					//gl::DrawArrays(gl::POINTS, 0, self.vert_len);
					gl::DrawElements(gl::TRIANGLES, self.len, gl::UNSIGNED_SHORT, null());
				},
				None => gl::DrawArrays(gl::TRIANGLES, 0, self.len),
			}
		}
	}
}
