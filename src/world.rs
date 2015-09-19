use DT;
use entity::{Entity, Camera};
use sdl2::keyboard::{Keycode, Mod};
use render::Render;

#[derive(Clone)]
pub struct World {
	/// The camera information
	pub camera: Camera,
	/// All of the entities in the world.
	pub entities: Vec<Entity>,
}

impl World {
	pub fn new(cam: Camera) -> World {
		World {
			camera: cam,
			entities: Vec::new()
		}
	}
	
	pub fn tick(&mut self, dt: DT) {
		for ent in self.entities.iter_mut() {
			ent.tick(dt);
		}
	}
	
	pub fn render(&self, ren: &mut Render) {
		ren.set_camera(&self.camera);
		for ent in self.entities.iter() {
			ent.render(ren);
		}
	}
	
	pub fn handle_keydown(&mut self, key: &Keycode, keymod: &Mod, repeat: bool) {
		let (_, _, _) = (key, keymod, repeat);
	}
	
	pub fn handle_keyup(&mut self, key: &Keycode, keymod: &Mod) {
		let (_, _) = (key, keymod);
	}
}
