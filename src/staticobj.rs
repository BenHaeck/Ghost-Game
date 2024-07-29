use macroquad::prelude::*;

use crate::collision::*;

use crate::options::*;

use crate::go_helpers::*;

const TRAP_ANIM_TIME: f32 = 0.5;

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
			TriggerType::FlyBox => draw_centered_texture(&texture, self.coll.pos, WHITE, DrawTextureParams{
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