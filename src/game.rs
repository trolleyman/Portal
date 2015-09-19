
use DT;
use world::World;
use render::Render;

use sdl2;
use sdl2::event::Event;

mod Key {
	pub use sdl2::keyboard::Keycode::*;
}

pub struct Game {
	worlds: [World; 2],
	next_index: usize,
	current_index: usize,
	to_quit: bool,
	paused: bool,
}
impl Game {
	pub fn new(world: World) -> Game {
		Game {
			worlds: [world.clone(), world],
			next_index: 1,
			current_index: 0,
			to_quit: false,
			paused: false,
		}
	}
	
	pub fn should_quit(&self) -> bool {
		self.to_quit
	}
	pub fn is_paused(&self) -> bool {
		self.paused
	}
	
	pub fn handle_events(&mut self, pump: &mut sdl2::EventPump) {
		for event in pump.poll_iter() {
			match event {
				Event::Quit{..} => {
					self.to_quit = true;
					break;
				},
				Event::KeyDown{ keycode:key, keymod, repeat, .. } => {
					if key.is_some() {
						match key.unwrap() {
							Key::Escape => {
								self.toggle_paused();
							},
							_ => {}
						}
						if !self.paused {
							self.worlds[self.next_index].handle_keydown(&key.unwrap(), &keymod, repeat);
						}
					}
				},
				Event::KeyUp{ keycode:key, keymod, .. } => {
					if key.is_some() && !self.paused {
						self.worlds[self.next_index].handle_keyup(&key.unwrap(), &keymod);
					}
				},
				Event::MouseMotion{xrel:x, yrel:y, ..} => {
					if !self.paused {
						self.worlds[self.next_index].camera.rotate(x as f32, y as f32);
					}
				}
				_ => {}
			}
		}
	}
	
	fn toggle_paused(&mut self) {
		self.paused = !self.paused;
	}
	
	pub fn tick(&mut self, dt: DT) {
		self.worlds[self.next_index].tick(dt);
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
	
	pub fn render(&self, ren: &mut Render) {
		self.worlds[self.current_index].render(ren);
		ren.swap();
	}
}
