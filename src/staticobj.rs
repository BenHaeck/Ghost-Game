use std::default;

use macroquad::prelude::*;

use crate::collision::*;

use crate::options::*;

use crate::go_helpers::*;

pub const TRAP_ANIM_TIME: f32 = 0.5;

const ORIENT_DIR: Vec2 = vec2(0.7, 1.4);

pub struct TriggerMan {
	objs: Vec<(CollBox, TriggerType)>,
}

pub struct Trigger {
	pub coll: CollBox,
	pub trigger_type: TriggerType,
}

impl Trigger {
	pub fn new (coll: CollBox, trigger_type: TriggerType) -> Self {
		Self {coll: coll, trigger_type: trigger_type}
	}

	pub fn update(objs: &mut [Self], dt: f32) {
		for i in 0..objs.len() {
			match objs[i].trigger_type {
				TriggerType::Trap(time) => {
					objs[i].trigger_type = TriggerType::Trap(0.0f32.max(time-dt))
				}
				_ => {}
			}
		}
	}

	pub fn coll_check <FCB>(objs: &mut [Self], other: &CollBox, mut callback: FCB) where FCB: FnMut(&Trigger) -> bool {
		for i in 0..objs.len() {
			if objs[i].coll.check_intersection(&other) {
				if callback(&objs[i]) {
					match objs[i].trigger_type {
						TriggerType::Trap(time) => {
							if time < 0.01 {
								objs[i].trigger_type = TriggerType::Trap(TRAP_ANIM_TIME);
							}
						}
						_=>{}
					}
				}
			}
		}
	}

	pub fn draw (&self, texture: &Texture2D) {
		let flip = self.coll.pos.dot(ORIENT_DIR) % 2.0 < 1.2;
		match self.trigger_type {
			TriggerType::FlyBox => draw_centered_texture(&texture, self.coll.pos, FB_COLOR, DrawTextureParams{
				source: Some(Rect::new(0.0, 64.0, 48.0, 48.0)),
				dest_size: Some(Vec2::splat(48.0)),
				flip_x: flip,
				..DrawTextureParams::default()
			}),

			TriggerType::Spikes => draw_centered_texture(&texture, self.coll.pos, DANGER_COLOR, DrawTextureParams{
				source: Some(Rect::new(32.0, 112.0, 16.0, 16.0)),
				dest_size: Some(Vec2::splat(16.0)),
				flip_x: flip,
				..DrawTextureParams::default()
			}),

			TriggerType::Trap(time) => {
				draw_centered_texture(&texture, self.coll.pos, WHITE, DrawTextureParams{
					source: Some(Rect::new(if TRAP_ANIM_TIME * 0.5 > time {48.0} else {48.0+16.0}, 112.0, 16.0, 16.0)),
					dest_size: Some(Vec2::splat(16.0)),
					flip_x: flip,
					..DrawTextureParams::default()
				});

				let time = 1.0-(time/TRAP_ANIM_TIME);
				let mut color = WHITE;
				color.a = 1.0-time*time;
				
				draw_centered_texture(&texture, self.coll.pos, color, DrawTextureParams {
					source: Some(Rect::new(80.0, 112.0, 16.0, 16.0)),
					dest_size: Some(Vec2::splat(96.0*time)),
					..DrawTextureParams::default()
				});
			},

			_ => {}
		}
	}
}

pub enum TriggerType {
	FlyBox,
	Trap (f32),
	Spikes,
}

const BLOCK_FADE: f32 = 0.3;
const BLOCK_KILL_TIME: f32 = 0.1;
const BLOCK_TIME: f32 = 1.2;

pub struct GhostBlocks {
	blocks: Vec<CollBox>,
	pub dim: Vec2,
	hit_timer: f32,
	anim_time: f32,
	anim_dir: f32
}

impl GhostBlocks {
	pub fn new (block_size: Vec2) -> Self {
		Self{blocks: Vec::new(), dim: block_size, hit_timer: 0.0, anim_time: 0.2, anim_dir: -1.0}
	}

	pub fn add (&mut self, pos: Vec2) {
		self.blocks.push(collbox!(pos, self.dim));
	}

	pub fn clear(&mut self) {
		self.blocks.clear();
	}

	pub fn update (&mut self, nm_triggered: bool, dt: f32) {
		self.anim_dir = if nm_triggered {
			self.hit_timer = BLOCK_TIME;
			1.0
		} else if self.hit_timer > 0.0 {
			1.0
		} else {
			-1.0
		};
		self.hit_timer = 0.0f32.max(self.hit_timer - dt);

		self.anim_time = (self.anim_time + dt*self.anim_dir).max(0.0).min(BLOCK_FADE);
	}

	pub fn draw(&self, texture: &Texture2D) {
		for i in 0..self.blocks.len() {
			let mut anim_time = self.anim_time / BLOCK_FADE;
			anim_time += anim_time * anim_time;
			anim_time *= 0.5;
			draw_centered_texture(&texture, self.blocks[i].pos, lerp_color(Color::new(0.5, 0.5, 0.5, 1.0), Color::new(0.4, 0.4, 0.4, 0.25), anim_time), DrawTextureParams{
				dest_size: Some(Vec2::splat(16.0)),
				source: Some(Rect::new(80.0, 96.0, 16.0, 16.0)),
				..DrawTextureParams::default()
			});
		}
	} 

	pub fn get_block_effect (&self) -> GhostBlockEffect{
		if self.anim_time <= 0.0 {return GhostBlockEffect::Collide;}
		if self.anim_dir < 0.0 && self.anim_time < BLOCK_KILL_TIME {return GhostBlockEffect::Kill;}
		GhostBlockEffect::PassThrough
	}

	pub fn get_block_slice<'a> (&'a self) -> &'a [CollBox] {
		&self.blocks
	}


}

#[derive(PartialEq)]
pub enum GhostBlockEffect {
	Collide,
	Kill,
	PassThrough
}