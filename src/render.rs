use prelude::*;

use entity::Camera;

use std::fs::File;
use std::io::prelude::*;
use std::ops::Drop;
use std::mem;
use std::sync::Arc;
use std::ptr::null;
use std::ffi::CString;

use na;

use sdl2::video::{GLContext, Window};

use gl::{self};
use gl::types::*;

pub struct Render<'a> {
	pub win: &'a mut Window,
	pub gl_context: &'a mut GLContext,
	pub main_shader: Shader,
	pub vp_mat: Mat4,
	pub m_mat: Mat4,
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
		};
		unsafe {
			gl::Enable(gl::CULL_FACE);
			gl::Enable(gl::DEPTH_TEST);
			gl::DepthFunc(gl::LESS);
		}
		ren.win.subsystem().gl_set_swap_interval(1);
		
		ren
	}
	
	pub fn swap(&mut self) {
		self.win.show();
		self.win.gl_swap_window();
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.1, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::Clear(gl::DEPTH_BUFFER_BIT);
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
	
	pub fn update_size(&mut self) {
		let (w, h) = self.win.drawable_size();
		unsafe {
			gl::Viewport(0, 0, w as i32, h as i32);
		}
	}
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
			
			let mut isLinked: GLint = 0;
			gl::GetProgramiv(prog, gl::LINK_STATUS, &mut isLinked);
			if isLinked == gl::FALSE as GLint {
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
			
			let mut isCompiled: GLint = 0;
			gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut isCompiled);
			if isCompiled == gl::FALSE as GLint {
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
	indices: Vec<na::Pnt3<usize>>,
	uvs: Vec<Pnt2>,
	normals: Vec<Vec3>
}
impl MeshBuilder {
	pub fn new() -> MeshBuilder {
		MeshBuilder {
			verts: Vec::new(),
			indices: Vec::new(),
			uvs: Vec::new(),
			normals: Vec::new(),
		}
	}
	
	pub fn vertex(&mut self, vert: Pnt3) {
		self.verts.push(vert);
	}
	pub fn index(&mut self, index: na::Pnt3<usize>) {
		self.indices.push(index);
	}
	pub fn uv(&mut self, uv: Pnt2) {
		self.uvs.push(uv);
	}
	pub fn normal(&mut self, normal: Vec3) {
		self.normals.push(normal);
	}
	
	pub fn finish(self) -> Mesh {
		let inds = if self.indices.len() == 0 {
			None
		} else {
			Some(Arc::new(self.indices))
		};
		let uvs = if self.uvs.len() == 0 {
			None
		} else {
			Some(Arc::new(self.uvs))
		};
		let norms = if self.normals.len() == 0 {
			None
		} else {
			Some(Arc::new(self.normals))
		};
		Mesh::new(Arc::new(self.verts), inds, uvs, norms)
	}
}

// This makes it so that Meshes leak during the program as entities die, but it does speed up the process of cloning the world
#[derive(Clone)]
pub struct Mesh {
	vao: GLuint,
	len: GLsizei,
	verts: GLuint,
	indices: GLuint,
	uvs: Option<GLuint>,
	normals: Option<GLuint>,
	mesh: TriMesh,
}
impl Mesh {
	pub fn new(vertices: Arc<Vec<Pnt3>>,
		       indices: Option<Arc<Vec<na::Pnt3<usize>>>>,
		       uvs: Option<Arc<Vec<Pnt2>>>,
		       normals: Option<Arc<Vec<Vec3>>>) -> Mesh {
		let inds = match indices {
			Some(inds) => {
				inds
			},
			None => {
				let inds = Vec::new();
				
				for i in (0..vertices.len()) {
					inds.push(na::Pnt3::new(i * 3, i * 3 + 1, i * 3 + 2));
				}
				Arc::new(inds)
			}
		};
		let mesh = TriMesh::new(vertices, inds, uvs, normals);
		
		Mesh::from_nc_mesh(mesh)
	}
	pub fn from_nc_mesh(mesh: TriMesh) -> Mesh {
		unsafe {
			let verts = mesh.vertices();
			let inds = mesh.indices();
			let uvs = mesh.uvs();
			let norms = mesh.normals();
			
			let mut vao: GLuint = 0;
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			
			let mut vert_vbo = 0;
			let mut inds_vbo = 0;
			gl::GenBuffers(1, &mut vert_vbo);
			gl::GenBuffers(1, &mut inds_vbo);
			
			// Specify that the data to be pushed is the verts
			gl::BindBuffer(gl::ARRAY_BUFFER, vert_vbo);
			// Push the verts to the GPU
			gl::BufferData(gl::ARRAY_BUFFER, (verts.len() * mem::size_of::<Pnt3>()) as i64, mem::transmute(verts.as_ptr()), gl::STATIC_DRAW);
			// Specify that it is attribute 0
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
			gl::EnableVertexAttribArray(0);
			
			// Now the indices
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, inds_vbo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (inds.len() * mem::size_of::<na::Pnt3<usize>>()) as i64, mem::transmute(inds.as_ptr()), gl::STATIC_DRAW);
			
			// Now the uvs
			let mut uv_vbo = 0;
			gl::GenBuffers(1, &mut uv_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, uv_vbo);
			match uvs {
				&Some(ref uvs) => {
					gl::BufferData(gl::ARRAY_BUFFER, (uvs.len() * mem::size_of::<Pnt2>()) as i64, mem::transmute(uvs.as_ptr()), gl::STATIC_DRAW);
				},
				&None => {}
			}
			gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, null());
			gl::EnableVertexAttribArray(1);
			
			// Now the normals
			let mut norm_vbo = 0;
			gl::GenBuffers(1, &mut norm_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, norm_vbo);
			match norms {
				&Some(ref norms) => {
					gl::BufferData(gl::ARRAY_BUFFER, (norms.len() * mem::size_of::<Vec3>()) as i64, mem::transmute(norms.as_ptr()), gl::STATIC_DRAW);
				},
				&None => {}
			}
			gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, null());
			gl::EnableVertexAttribArray(2);
			
			// So that we copy the stuff over before they are freed...
			gl::Flush();
			
			Mesh {
				vao: vao,
				len: inds.len() as GLsizei,
				verts: vert_vbo,
				indices: inds_vbo,
				uvs: if uvs.is_some() { Some(uv_vbo) } else { None },
				normals: if norms.is_some() { Some(norm_vbo) } else { None },
				mesh: mesh,
			}
		}
	}
	
	pub fn new_triangle(scale: f32) -> Mesh {
		Mesh::new(Arc::new(vec![
			Vec3::new(-0.5,  0.1, -1.0) * scale,
			Vec3::new( 0.0,  1.1, -1.0) * scale,
			Vec3::new( 0.5,  0.1, -1.0) * scale,
		]), Arc::new(vec![
			
		]), Arc::new(vec![
			
		]), Arc::new(vec![
			
		]))
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
				mb.push(Vec3::new((x as f32      ) * (w / num_w as f32) - offset_x, 0.0, (y as f32      ) * (h / num_h as f32) - offset_y), col);
				mb.push(Vec3::new((x as f32 + 1.0) * (w / num_w as f32) - offset_x, 0.0, (y as f32      ) * (h / num_h as f32) - offset_y), col);
				mb.push(Vec3::new((x as f32 + 1.0) * (w / num_w as f32) - offset_x, 0.0, (y as f32 + 1.0) * (h / num_h as f32) - offset_y), col);
				
				mb.push(Vec3::new((x as f32      ) * (w / num_w as f32) - offset_x, 0.0, (y as f32      ) * (h / num_h as f32) - offset_y), col);
				mb.push(Vec3::new((x as f32 + 1.0) * (w / num_w as f32) - offset_x, 0.0, (y as f32 + 1.0) * (h / num_h as f32) - offset_y), col);
				mb.push(Vec3::new((x as f32      ) * (w / num_w as f32) - offset_x, 0.0, (y as f32 + 1.0) * (h / num_h as f32) - offset_y), col);
			}
		}
		mb.finish()
	}
	
	pub fn render(&self, ren: &mut Render, model_mat: Mat4) {
		unsafe {
			ren.set_model_mat(model_mat);
			ren.main_shader.use_prog();
			
			gl::BindVertexArray(self.vao);
			gl::DrawElements(gl::TRIANGLES, self.len, gl::FLOAT, null());
		}
	}
}
