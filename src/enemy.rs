use std::clone;

use crate::{go_helpers::*, player};
use macroquad::prelude::*;
use crate::collision::*;

use crate::options::{DANGER_COLOR};


const AG_SPEED: f32 = 94.0*0.5;
const ENEMY_SIZE: Vec2 = vec2(16.0, 16.0);

const TURRET_FIRE_RECOV: f32 = 1.5;
const TURRET_SHOOT_LENGTH: f32 = 1.0;

const TURRET_BULLET_SPEED: f32 = 100.0;

const ENEMY_DET_SIZE: f32 = 48.0;

const BULLET_SIZE:f32 = 2.0;

pub struct Enemy {
	pub cbox: CollBox,
	pub enemy_type: EnemyType,
}


impl Enemy {
	pub fn new(pos: Vec2, entype: EnemyType) -> Self {
		Self {
			cbox: collbox!(pos, Vec2::splat(4.0)),
			enemy_type: entype,
		}
	}


	pub fn update(&mut self, player_box: &CollBox, bullets: &mut Vec<Bullet>, trap_triggered: bool, dt: f32) {
		let player_in_range = player_box.to_circle().check_intersection(&collcircle!(self.cbox.pos, ENEMY_DET_SIZE)) || trap_triggered;
		
		let dir = (player_box.pos - self.cbox.pos).normalize_or_zero();
		match self.enemy_type {
			EnemyType::AngryGhosts(mut active) =>{
				active = active||player_in_range;

				if active {

					self.cbox.pos += dir * AG_SPEED * dt;
				}
				self.enemy_type = EnemyType::AngryGhosts(active);
			}

			EnemyType::Turret(mut recov, mut flip_x) => {
				recov = f32::max(recov - dt, 0.0);

				if recov <= 0.0 && player_in_range {
					bullets.push(Bullet::new(collcircle!(self.cbox.pos, BULLET_SIZE), dir * TURRET_BULLET_SPEED, Some(10.0)));
					recov = TURRET_FIRE_RECOV;
				}
				self.enemy_type = EnemyType::Turret(recov, dir.x < 0.0);
			}

			_ => {}
		}
	}

	pub fn draw(&self, texture: &Texture2D) {
		//draw_rectangle(draw_loc.x, draw_loc.y, self.cbox.half_dim.x * 2.0, self.cbox.half_dim.y * 2.0, RED);
		match self.enemy_type {
			EnemyType::AngryGhosts(active)=> {
				draw_centered_texture(&texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
					dest_size: Some(ENEMY_SIZE),
					source: Some(Rect::new(if active {16.0} else {0.0}, 0.0, 16.0, 16.0)),
					..DrawTextureParams::default()
				})
			}

			EnemyType::Turret(recov, flip_x) => {
				draw_centered_texture(texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
					source: Some(Rect::new(if recov < TURRET_SHOOT_LENGTH {0.0} else {16.0}, 32.0, 16.0, 16.0)),
					dest_size: Some(Vec2::splat(16.0)),
					flip_x: flip_x,
					..DrawTextureParams::default()
				});
			}
			_ => {}
		}
	}

	pub fn get_friendly_fire (&self) -> bool {
		match self.enemy_type {
			EnemyType::AngryGhosts(_) => true,
			_ => false,
		}
	}
}

pub fn new_angry_ghost(pos:Vec2) -> Enemy {
	Enemy::new(pos, EnemyType::AngryGhosts(false))
}

pub fn new_turret(pos: Vec2) -> Enemy {
	Enemy::new(pos, EnemyType::Turret(0.0, false))
}

pub enum EnemyType {
	AngryGhosts(bool),
	Turret(f32, bool)
}

#[derive(Clone)]
pub struct Bullet {
	pub coll: CollCircle,
	pub motion: Vec2,
	pub life_time: Option<f32>,
}

impl Bullet {
	pub fn new(coll: CollCircle, motion: Vec2, life_time: Option<f32>) -> Self {
		Self {
			coll: coll,
			motion:motion,
			life_time: life_time
		}
	}

	pub fn update(&mut self, dt:f32) {
		self.coll.pos += self.motion * dt;

	}

	pub fn life_time_update (&mut self, dt:f32) -> bool {
		if let Some(lt) = self.life_time {
			let next_lt = lt - dt;
			self.life_time = Some(next_lt);
			if next_lt < 0.0 {
				true
			}
			else {
				false
			}
		} else {
			false
		}
	}
}