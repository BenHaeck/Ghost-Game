use macroquad::prelude::*;

use crate::collision::*;
use crate::go_helpers::*;

pub struct WallMan<TTile> where TTile: Clone{
	pub wall_colliders: Vec<MultiCollider<CollBox>>,
	wall_sprites: Vec<(Vec2, TTile)>,

	placeholder: TTile
}

impl <TTile> WallMan<TTile> where TTile:Clone{
	pub fn new(wall_mc: Vec<MultiCollider<CollBox>>, tile_placeholder: TTile) -> Self {
		Self{
			wall_colliders: wall_mc,
			wall_sprites: Vec::new(),
			placeholder: tile_placeholder
		}		
	}

	pub fn add_wall(&mut self, rect: CollBox) {
		MultiCollider::add_if_intersect_slice(self.wall_colliders.as_mut_slice(), &rect);
		self.wall_sprites.push((rect.pos, self.placeholder.clone()));
	}

	pub fn assign_tile_types <F>(&mut self, f:F, check_dist: Vec2, coll_dim: Vec2, world_dim: Option<Vec2>) where F: Fn(IVec2, BVec2)-> TTile {
		for i in 0..self.wall_sprites.len() {
			let mut exposure = ivec2(0, 0);
			let mut collided = BVec2::new(false, false);
			for j in [-1.0, 1.0].iter() {
				let (pos, _) = self.wall_sprites[i].clone();
				let check_pos = pos + vec2(check_dist.x * j.clone(), 0.0);
				let in_world = if let Some(dim) = world_dim {
					0.0 <= check_pos.x && check_pos.x <= dim.x
				}else {true};
				if MultiCollider::check_intersection_slice(self.wall_colliders.as_slice(), &collbox!(check_pos, coll_dim)) || !in_world {
					exposure.x += j.clone() as i32;
					collided.x = true;
				}

				let check_pos: Vec2 = pos + vec2(0.0, check_dist.y * j.clone());
				let in_world = if let Some(dim) = world_dim {
					0.0 <= check_pos.y && check_pos.y <= dim.y
				}else {true};
				if MultiCollider::check_intersection_slice(self.wall_colliders.as_slice(), &collbox!(check_pos, coll_dim))  || !in_world {
					exposure.y += j.clone() as i32;
					collided.y = true;
				}
			}
			self.wall_sprites[i] = (self.wall_sprites[i].0, f(exposure, collided));
		}
	}

	pub fn cull(&mut self) {
		println!("Before cull: {}", self.wall_colliders.len());
		for i in (0..self.wall_colliders.len()).rev() {
			if self.wall_colliders[i].read_multi().len() <= 0 {
				self.wall_colliders.remove(i);
			}
		}
		
		println!("After cull: {}", self.wall_colliders.len());
	}

	pub fn clear (&mut self) {
		for i in 0..self.wall_colliders.len() {
			self.wall_colliders[i].clear();
		}
		self.wall_sprites.clear();
	}

	pub fn draw_walls <F> (&self, f: F) where F: Fn(Vec2, &TTile) {
		for i in 0..self.wall_sprites.len() {
			let (pos, tile_type) = &self.wall_sprites[i];
			f(pos.clone(), tile_type);
		}
	}
}

