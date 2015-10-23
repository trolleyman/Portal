extern crate ncollide as nc;
extern crate nalgebra as na;
//extern crate rand;
extern crate sdl2;
extern crate gl;

pub mod world;
pub mod entity;
pub mod game;
pub mod render;

use render::{Render, Mesh};
use world::World;
use game::Game;
use entity::{Entity, Camera, Portal};

use sdl2::Sdl;
use sdl2::keyboard::KeyboardState;

pub type DT = f32;

pub mod prelude {
	pub use na::{
		Absolute, Cast, Col, Cross, Det, Dot, Eye, FromHomogeneous, Inv, Norm, PntAsVec, Rotate, Rotation, Row,
		ToHomogeneous, RotationTo, Transform, Transformation, Translate, Translation,
	};
	pub use {translation_mat, get_rotation_between};
	#[allow(non_snake_case)]
	pub mod Key {
		pub use sdl2::keyboard::Keycode::*;
	}
	#[allow(non_snake_case)]
	pub mod Scan {
		pub use sdl2::keyboard::Scancode::*;
	}
	
	pub use DT;
	pub use {Mat3, Mat4, Ortho3, Persp3, Pnt2, Pnt3, Quat, Rot2, Rot3, UnitQuat, Vec2, Vec3, Vec4};
	pub use {TriMesh};
}
pub type TriMesh = nc::shape::TriMesh<na::Pnt3<f32>>;

pub type Mat3 = na::Mat3<f32>;
pub type Mat4 = na::Mat4<f32>;
pub type Ortho3 = na::Ortho3<f32>;
pub type Persp3 = na::Persp3<f32>;
pub type Pnt2 = na::Pnt2<f32>;
pub type Pnt3 = na::Pnt3<f32>;
pub type Quat = na::Quat<f32>;
pub type Rot2 = na::Rot2<f32>;
pub type Rot3 = na::Rot3<f32>;
pub type UnitQuat = na::UnitQuat<f32>;
pub type Vec2 = na::Vec2<f32>;
pub type Vec3 = na::Vec3<f32>;
pub type Vec4 = na::Vec4<f32>;

pub fn translation_mat(t: &Vec3) -> Mat4 {
	Mat4::new(1.0, 0.0, 0.0, t.x,
	          0.0, 1.0, 0.0, t.y,
	          0.0, 0.0, 1.0, t.z,
	          0.0, 0.0, 0.0, 1.0)
}

pub fn get_rotation_between(mut a: Vec3, mut b: Vec3) -> Mat4 {
	use na::{Norm, Cross, Dot, ToHomogeneous};
	
	a = a.normalize();
	b = b.normalize();
	
	let u = a.cross(&b); // axis
	let cos = a.dot(&b); // cos(angle)
	let ncos = 1.0 - cos;
	let sin = a.norm(); // sin(angle)
	
	// Taken from wikipedia
	Mat3::new(cos + u.x*u.x*ncos, u.x*u.y*ncos - u.z*sin, u.x*u.z*ncos + u.y*sin,
	          u.y*u.z*ncos + u.z*sin, cos + u.y*u.y*ncos, u.y*u.z*ncos - u.z*sin,
	          u.z*u.x*ncos - u.y*sin, u.z*u.y*ncos + u.x*sin, cos + u.z*u.z*ncos).to_homogeneous()
}

fn main() {
	let sdl = match sdl2::init() {
		Ok(sdl) => sdl,
		Err(s)  => panic!("sdl init error: {}", &s)
	};
	let video = match sdl.video() {
		Ok(sub) => sub,
		Err(s)  => panic!("sdl video subsystem init error: {}", &s)
	};
	let gl_attr = video.gl_attr();
	gl_attr.set_stencil_size(8);
	
	let mut timer = match sdl.timer() {
		Ok(sub) => sub,
		Err(s)  => panic!("sdl timer subsystem init error: {}", &s)
	};
	let mut win = match video.window("Portal", 800, 600).allow_highdpi().resizable().hidden().opengl().position_centered().build() {
		Ok(win) => win,
		Err(s)  => panic!("sdl window init error: {}", &s),
	};
	let mut context = match win.gl_create_context() {
		Ok(c)  => c,
		Err(s) => panic!("sdl opengl context creation error: {}", &s),
	};
	win.gl_make_current(&context).unwrap();
	let mut pump = match sdl.event_pump() {
		Ok(sub) => sub,
		Err(s)  => panic!("sdl event subsystem init error: {}", &s),
	};
	sdl.mouse().set_relative_mouse_mode(true);
	
	gl::load_with(|name| video.gl_get_proc_address(name));
	
	let mut ren = Render::new(&mut win, &mut context);
	
	let mut init_world = World::new(Camera::new(Vec3::new(0.0, 1.0, 0.0), 90.0));
	init_world.entities.push(Entity::new(Vec3::new(-0.3, 0.6, 0.6), Vec3::new(0., 0./*5*/, 0./*1*/), Mesh::new_triangle(0.5)));
	init_world.entities.push(Entity::new(Vec3::new( 0.3, 0.6, 0.6), Vec3::new(0., 0./*5*/, 0./*1*/), Mesh::new_square(0.5)));
	let planes = Mesh::new_planes(10, 10, 10.0, 10.0, Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0));
	init_world.entities.push(Entity::new_static(Vec3::new(0.0, 0.0, 0.0), planes));
	init_world.set_portals(
		Portal::new(Vec3::new(0.0, 1.0, 4.0), Vec3::new(0.0, 0.0, -1.0), 0.9, 1.4),
		Portal::new(Vec3::new(-1.5, 1.0, 2.5), Vec3::new(1.0, 0.0, 0.0), 0.9, 1.4)
	);
	
	let mut game = Game::new(init_world, sdl.mouse());
	main_loop(&sdl, &mut timer, &mut pump, &mut game, &mut ren);
}

fn main_loop(sdl: &Sdl, timer: &mut sdl2::TimerSubsystem, pump: &mut sdl2::EventPump, game: &mut Game, ren: &mut Render) {
	let mut total: u32 = 0;
	let mut prev = timer.ticks();
	let mut marker = prev;
	let mut frames_since_marker = 0;
	loop {
		let now = timer.ticks();
		let dur = now - prev;
		if now as i64 - marker as i64 >= 1000 {
			game.set_fps(frames_since_marker);
			marker = now;
			frames_since_marker = 0;
		}
		prev = now;
		
		let dt: DT = dur as DT / 1_000.0;
		/*let secs: u64 = dur.as_secs();
		let nsecs: u32 = dur.subsec_nanos();
		let dt: DT = (secs as DT) + ((nsecs as DT) / 1_000_000_000.0);*/
		if !game.is_paused() {
			total += dur;
		}
		
		print!("fps:{: >3}, total: {: >7.3}s, dt: {: >.3}s", game.get_fps(), (total as f64) / 1_000.0, dt);
		print!(" --- ");
		game.get_current_world().print();
		print!(" --- ");
		ren.print();
		println!("");
		
		if !game.is_paused() {
			game.tick(dt, &KeyboardState::new(&pump));
		}
		game.handle_events(sdl, pump, ren);
		game.swap();
		// This render can be done by a seperate thread. Probably.
		game.render(ren);
		frames_since_marker += 1;
		
		if game.should_quit() {
			println!("quitting...");
			break;
		}
	}
}
