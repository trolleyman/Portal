use prelude::*;

use world::World;
use render::Render;

use sdl2;
use sdl2::Sdl;
use sdl2::event::{Event, WindowEventId};
use sdl2::keyboard::KeyboardState;
use sdl2::mouse::MouseUtil;

pub struct Game {
	worlds: [World; 2],
	next_index: usize,
	current_index: usize,
	to_quit: bool,
	paused: bool,
	mouse_util: MouseUtil,
	fps: u32,
}
impl Game {
	pub fn new(world: World, mouse_util: MouseUtil) -> Game {
		Game {
			worlds: [world.clone(), world],
			next_index: 1,
			current_index: 0,
			to_quit: false,
			paused: false,
			mouse_util: mouse_util,
			fps: 0,
		}
	}
	
	pub fn get_current_world<'a>(&'a self) -> &'a World {
		&self.worlds[self.current_index]
	}
	
	pub fn should_quit(&self) -> bool {
		self.to_quit
	}
	pub fn is_paused(&self) -> bool {
		self.paused
	}
	
	pub fn handle_events(&mut self, _sdl: &Sdl, pump: &mut sdl2::EventPump, ren: &mut Render) {
		for event in pump.poll_iter() {
			match event {
				Event::Quit{..} => {
					self.to_quit = true;
					break;
				},
				Event::KeyDown{ keycode:key, keymod, repeat, .. } => {
					match key {
						Some(Key::Escape) => {
							self.toggle_paused();
						},
						Some(Key::F7) => {
							ren.toggle_wireframes();
						},
						Some(Key::F8) => {
							ren.toggle_portal_rendering();
						},
						_ => {}
					}
					if !self.paused {
						self.worlds[self.next_index].handle_keydown(&key.unwrap(), &keymod, repeat);
					}
				},
				Event::KeyUp{ keycode:key, keymod, .. } => {
					if key.is_some() && !self.paused {
						self.worlds[self.next_index].handle_keyup(&key.unwrap(), &keymod);
					}
				},
				Event::MouseMotion{xrel:x, yrel:y, ..} => {
					if !self.paused {
						self.worlds[self.next_index].handle_mouse_motion(x as f32, y as f32);
					}
				},
				Event::MouseButtonDown{ mouse_btn, .. } => {
					if self.paused && mouse_btn == sdl2::mouse::Mouse::Left {
						self.toggle_paused();
					}
				},
				Event::Window{ win_event_id, .. } => {
					match win_event_id {
						WindowEventId::SizeChanged => {
							ren.update_size();
						},
						_ => {}
					}
				},
				_ => {}
			}
		}
	}
	
	fn toggle_paused(&mut self) {
		self.paused = !self.paused;
		self.mouse_util.set_relative_mouse_mode(!self.paused);
	}
	
	pub fn tick(&mut self, dt: DT, state: &KeyboardState) {
		self.worlds[self.next_index].tick(dt, state);
	}
	
	pub fn swap(&mut self) {
		self.current_index = self.next_index;
		self.next_index = if self.next_index == 0 {
			1
		} else {
			0
		};
		let current_world = self.worlds[self.current_index].clone();
		self.worlds[self.next_index] = current_world;
	}
	
	pub fn set_fps(&mut self, fps: u32) {
		self.fps = fps;
	}
	
	pub fn get_fps(&self) -> u32 {
		self.fps
	}
	
	pub fn render(&self, ren: &mut Render) {
		self.worlds[self.current_index].render(ren);
		ren.swap();
	}
}
