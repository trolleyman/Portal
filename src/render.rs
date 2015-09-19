
use {Mat4, Vec3};
use entity::Camera;

use std::fs::File;
use std::io::prelude::*;
use std::ops::Drop;
use std::mem;
use std::ptr::null;

use cg;

use sdl2::video::{GLContext, Window};

use gl::{self};
use gl::types::*;

pub struct Render<'a> {
	pub win: &'a mut Window,
	pub gl_context: &'a GLContext,
	pub main_shader: Shader,
	pub vp_mat: Mat4,
}

impl<'a> Render<'a> {
	pub fn new(win: &'a mut Window, context: &'a GLContext) -> Render<'a> {
		//let _ = win.gl_set_context_to_current();
		let ren = Render {
			win: win,
			gl_context: context,
			main_shader: match Shader::from_files("shaders/main.vs", "shaders/main.fs") {
					Ok(s) => s,
					Err(e) => panic!("{}", e),
				},
			vp_mat: Mat4::identity(),
		};
		ren.win.subsystem().gl_set_swap_interval(1);
		
		ren
	}
	
	pub fn swap(&mut self) {
		self.win.show();
		self.win.gl_swap_window();
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
	}
	
	pub fn set_camera(&mut self, cam: &Camera) {
		// Recalculate VP matrix
		//let projection = cg::projection;
		//let view = cam.view;
		//self.vp_mat = projection * view;
	}
	
	pub fn set_model_mat(&mut self, mat: &Mat4) {
		
	}
}

pub struct Shader {
	prog: GLuint,
	vs: GLuint,
	fs: GLuint,
	//mvp_pos: GLsomething,
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
	pub fn from_strs(vs: &[u8], fs: &[u8]) -> Result<Shader, String> {
		unsafe {
			let s = Shader {
				prog: gl::CreateProgram(),
				vs: gl::CreateShader(gl::VERTEX_SHADER),
				fs: gl::CreateShader(gl::FRAGMENT_SHADER),
			};
			
			try!(Shader::compile_shader(s.vs, vs));
			try!(Shader::compile_shader(s.fs, fs));
			
			gl::AttachShader(s.prog, s.vs);
			gl::AttachShader(s.prog, s.fs);
			
			gl::BindAttribLocation(s.prog, 0, "in_pos".as_ptr() as *const i8);
			gl::BindAttribLocation(s.prog, 1, "in_color".as_ptr() as *const i8);
			
			gl::LinkProgram(s.prog);
			
			let mut isLinked: GLint = 0;
			gl::GetProgramiv(s.prog, gl::LINK_STATUS, &mut isLinked);
			if isLinked == gl::FALSE as GLint {
				let mut len: GLint = 0;
				gl::GetProgramiv(s.prog, gl::INFO_LOG_LENGTH, &mut len);
				
				let mut log: Vec<u8> = Vec::with_capacity(len as usize);
				gl::GetProgramInfoLog(s.prog, len, &mut len, log.as_mut_ptr() as *mut i8);
				log.set_len(len as usize);
				
				let mut s = String::from("error linking program: \n");
				s.push_str(&String::from_utf8_lossy(&log));
				return Err(s);
			}
			
			s.use_prog();
			
			Ok(s)
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
	verts: Vec<Vec3>,
	colors: Vec<Vec3>,
}
impl MeshBuilder {
	pub fn new() -> MeshBuilder {
		MeshBuilder {
			verts: Vec::new(),
			colors: Vec::new()
		}
	}
	
	pub fn push(&mut self, vert: Vec3, color: Vec3) {
		self.verts.push(vert);
		self.colors.push(color);
	}
	
	pub fn finish(&self) -> Mesh {
		Mesh::new(&self.verts, &self.colors)
	}
}

// This makes it so that Meshes leak during the program as entities die, but it does speed up the process of cloning the world
#[derive(Debug, Copy, Clone)]
pub struct Mesh {
	vao: GLuint,
	len: GLsizei,
	verts: GLuint,
	colors: GLuint,
}
impl Mesh {
	pub fn new(verts: &[Vec3], colors: &[Vec3]) -> Mesh {
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
				len: verts.len() as GLsizei,
				verts: vbo[0],
				colors: vbo[1],
			}
		}
	}
	
	pub fn render(&self, ren: &mut Render, model_mat: &Mat4) {
		unsafe {
			ren.set_model_mat(model_mat);
			ren.main_shader.use_prog();
			
			gl::BindVertexArray(self.vao);
			gl::DrawArrays(gl::TRIANGLES, 0, self.len);
		}
	}
}