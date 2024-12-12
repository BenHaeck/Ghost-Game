use std::clone;
use std::f32::consts;


use crate::go_helpers::*;
use crate::player;
use macroquad::prelude::*;
use crate::collision::*;

use crate::options::*;

use crate::partical_system::*;


const AG_SPEED: f32 = 94.0*0.5 + 4.0;
const ENEMY_SIZE: Vec2 = vec2(16.0, 16.0);
const ANG_SPEED_BOOST:f32 = 1.7;
const AG_SPEED_CHANGE: f32 = 200.0;

const TURRET_FIRE_RECOV: f32 = 1.5;
const TURRET_SHOOT_LENGTH: f32 = 1.0;
const TURRET_SPEED_BOOST: f32 = 1.75;

const TURRET_BULLET_SPEED: f32 = 100.0;

pub const ENEMY_DET_SIZE: f32 = 60.0;

const BULLET_SIZE:f32 = 2.0;

const STALKER_SLOW_SPEED: f32 = 24.0;
const STALKER_FAST_SPEED: f32 = 94.0 * 3.0;
const STALKER_SPEED_CHANGE: f32 = 2.4;
const STALKER_TARGET_RADIUS: f32 = 64.0;
const STALKER_ANGER_TIMER: f32 = 0.3;

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


	pub fn update(&mut self, player_box: &CollBox, bullets: &mut Vec<Bullet>, in_fb: bool, trap_triggered: bool, dt: f32) {
		let player_in_range = player_box.to_circle().check_intersection(&collcircle!(self.cbox.pos, ENEMY_DET_SIZE)) || trap_triggered;
		
		let dir = (player_box.pos - self.cbox.pos).normalize_or_zero();
		match self.enemy_type {
			EnemyType::AngryGhosts(mut speed, mut active) =>{
				
				active = active||player_in_range;
				let target_speed = if !active {0.0} else { (if in_fb{ANG_SPEED_BOOST} else {1.0}) * AG_SPEED};

				speed = move_toward(speed, target_speed, AG_SPEED_CHANGE * dt);

				if active {

					self.cbox.pos += dir * speed * dt;
				}
				self.enemy_type = EnemyType::AngryGhosts(speed, active);
			}

			EnemyType::Turret(mut recov) => {
				recov = f32::max(recov - dt * (if in_fb{TURRET_SPEED_BOOST} else {1.0}), 0.0);

				if recov <= 0.0 && player_in_range {
					bullets.push(Bullet::new(collcircle!(self.cbox.pos, BULLET_SIZE), dir * TURRET_BULLET_SPEED, Some(10.0)));
					recov = TURRET_FIRE_RECOV;
				}
				self.enemy_type = EnemyType::Turret(recov);
			}

			EnemyType::StalkingGhost(mut velocity, mut anger_timer, mut agro) => {
				let mut used_speed= 0.0f32;
				agro = agro || player_in_range;
				if agro {
					// stalks the player
					if Vec2::distance_squared(self.cbox.pos, player_box.pos)
					> STALKER_TARGET_RADIUS * STALKER_TARGET_RADIUS{
						used_speed = STALKER_SLOW_SPEED
					}
					// anger ghost
					if trap_triggered {anger_timer = STALKER_ANGER_TIMER;}
					// chase player
					if anger_timer > 0.0 {
						used_speed = STALKER_FAST_SPEED;
					}
					// move ghost
					velocity = move_toward_ex_2D(velocity, (player_box.pos- self.cbox.pos).normalize_or_zero() * used_speed, dt * STALKER_SPEED_CHANGE);
					self.cbox.pos += velocity * dt;
				}
				self.enemy_type = EnemyType::StalkingGhost(velocity, (anger_timer-dt).max(-0.1), agro);
			}

			_ => {}
		}
	}

	pub fn draw(&self, texture: &Texture2D, player_pos: Vec2) {
		//draw_rectangle(draw_loc.x, draw_loc.y, self.cbox.half_dim.x * 2.0, self.cbox.half_dim.y * 2.0, RED);
		let flip_x =  self.cbox.pos.x > player_pos.x;
		match self.enemy_type {
			EnemyType::AngryGhosts(_, active)=> {
				draw_centered_texture(&texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
					dest_size: Some(ENEMY_SIZE),
					source: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
					flip_x: flip_x,
					..DrawTextureParams::default()
				});
				let eyes_offset = if active {(player_pos - self.cbox.pos).normalize_or_zero()} else {Vec2::ZERO} + vec2(0.0, -1.0);
				draw_centered_texture(&texture, self.cbox.pos + eyes_offset, DANGER_COLOR, DrawTextureParams{
					dest_size: Some(vec2(16.0, 8.0)),
					source: Some(Rect::new(16.0, if !active {8.0} else {0.0}, 16.0, 8.0)),
					..Default::default()
				});
			}

			EnemyType::Turret(recov) => {
				draw_centered_texture(texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
					source: Some(Rect::new(if recov < TURRET_SHOOT_LENGTH {0.0} else {16.0}, 32.0, 16.0, 16.0)),
					dest_size: Some(Vec2::splat(16.0)),
					flip_x: flip_x,
					..DrawTextureParams::default()
				});
			}

			EnemyType::StalkingGhost(vel,anger_time , agro) => {
				if anger_time <= 0.0 {
					draw_centered_texture(texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
						source: Some(Rect::new(if agro {0.0} else {16.0}, 48.0, 16.0, 32.0)),
						dest_size: Some(Vec2::new(16.0, 32.0)),
						flip_x: flip_x,
						..Default::default()
					});
				} else {
					let angle = 
					if vel.distance_squared(Vec2::ZERO) > 0.1 {
						f32::atan2(vel.y, vel.x)
					} else {
						let dist = player_pos - self.cbox.pos;
						f32::atan2(dist.x, dist.y)
					} + consts::PI/4.0;
					draw_centered_texture(texture, self.cbox.pos, DANGER_COLOR, DrawTextureParams{
						source: Some(Rect::new(0.0, 80.0, 32.0, 32.0)),
						dest_size: Some(vec2(32.0, 32.0)),
						rotation: angle,
						..Default::default()
					});
				}
			}

			_ => {
				draw_coll_box(self.cbox, RED);
			}
		}
	}

	pub fn get_friendly_fire (&self) -> bool {
		match self.enemy_type {
			EnemyType::AngryGhosts(_, _) => true,
			_ => false,
		}
	}
}

pub fn new_angry_ghost(pos:Vec2) -> Enemy {
	Enemy::new(pos, EnemyType::AngryGhosts(0.0, false))
}

pub fn new_turret(pos: Vec2) -> Enemy {
	Enemy::new(pos, EnemyType::Turret(0.0))
}

pub fn new_stalker(pos: Vec2) -> Enemy {
	let mut enemy = Enemy::new(pos, EnemyType::StalkingGhost(Vec2::ZERO, 0.0, false));
	enemy.cbox.half_dim = vec2(6.0, 6.0);
	enemy
}

pub fn create_enemy_death_particals(partical_system: &mut ParticalSystem, pos: Vec2) {
	partical_system.create_partical(13, pos, 7.0, pos, 16.0, 0.5, 1, ParticalRenderer::Circle);
}

pub enum EnemyType {
	AngryGhosts(f32, bool),
	Turret(f32),
	StalkingGhost(Vec2, f32, bool),
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