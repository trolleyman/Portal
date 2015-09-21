#![allow(non_snake_case)]
extern crate cgmath as cg;
extern crate sdl2;
extern crate gl;

pub mod world;
pub mod entity;
pub mod game;
pub mod render;

use render::{Render, Mesh};
use world::World;
use game::Game;
use entity::{Entity, Camera};

use sdl2::Sdl;
use sdl2::keyboard::KeyboardState;

pub mod prelude {
	pub use cg::{Vector, Zero, Rotation, Rotation3};
	pub mod Key {
		pub use sdl2::keyboard::Keycode::*;
	}
	pub mod Scan {
		pub use sdl2::keyboard::Scancode::*;
	}
	
	pub use {DT, Vec3, Mat4};
}

pub type DT = f32;
pub type Vec3 = cg::Vector3<f32>;
pub type Mat4 = cg::Matrix4<f32>;

fn main() {
	let sdl = match sdl2::init() {
		Ok(sdl) => sdl,
		Err(s)  => panic!("sdl init error: {}", &s)
	};
	let video = match sdl.video() {
		Ok(sub) => sub,
		Err(s)  => panic!("sdl video subsystem init error: {}", &s)
	};
	
	let mut timer = match sdl.timer() {
		Ok(sub) => sub,
		Err(s)  => panic!("sdl timer subsystem init error: {}", &s)
	};
	let mut win = match video.window("Portal", 800, 600).allow_highdpi().resizable().hidden().opengl().position_centered().build() {
		Ok(win) => win,
		Err(s)  => panic!("sdl window init error: {}", &s),
	};
	let context = match win.gl_create_context() {
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
	
	let mut ren = Render::new(&mut win, &context);
	
	let mut init_world = World::new(Camera::new(Vec3::new(0.0, 1.0, 0.0), 90.0));
	init_world.entities.push(Entity::new(Vec3::new(0.0, 0.5815, -0.4340), Vec3::new(0.0, 0./*5*/, 0./*1*/), Mesh::new_triangle(1.0)));
	let planes = Mesh::new_planes(10, 10, 10.0, 10.0, Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0));
	init_world.entities.push(Entity::new_static(Vec3::new(0.0, 0.0, 0.0), planes));
	
	let mut game = Game::new(init_world, sdl.mouse());
	main_loop(&sdl, &mut timer, &mut pump, &mut game, &mut ren);
}

fn main_loop(sdl: &Sdl, timer: &mut sdl2::TimerSubsystem, pump: &mut sdl2::EventPump, game: &mut Game, ren: &mut Render) {
	let mut total: DT = 0.0;
	let mut prev = timer.ticks();
	loop {
		let now = timer.ticks();
		let dur = now - prev;
		prev = now;
		
		let dt: DT = dur as DT / 1_000.0;
		/*let secs: u64 = dur.as_secs();
		let nsecs: u32 = dur.subsec_nanos();
		let dt: DT = (secs as DT) + ((nsecs as DT) / 1_000_000_000.0);*/
		if !game.is_paused() {
			total += dt;
		}
		
		print!("total: {: <.5}s, dt: {: <.5}s ", total, dt);
		game.get_current_world().print();
		
		if !game.is_paused() {
			game.tick(dt, &KeyboardState::new(&pump));
		}
		game.handle_events(sdl, pump);
		game.swap();
		// This render can be done by a seperate thread.
		game.render(ren);
		
		if game.should_quit() {
			println!("quitting...");
			break;
		}
	}
}
