
use crate::collision::*;
use crate::animation::*;
use crate::go_helpers::draw_centered_texture;
use crate::go_helpers::{move_toward, move_toward_2D, move_toward_ex_2D, lerp_color};
use crate::options::PLAYER_COLOR;
use crate::options::PLAYER_TIRED_COLOR;
use macroquad::prelude::*;

const PLAYER_SPEED: f32 = 96.0*0.7;
const PLAYER_SPEED_CHANGE: f32 = PLAYER_SPEED * 6.0;
const PLAYER_AIR_CHANGE: f32 = PLAYER_SPEED * 3.5;
const PLAYER_REVERSE_CHANGE: f32 = 3.0;

const PLAYER_FLIGHT_SPEED: f32 = 128.0;
const PLAYER_FLIGHT_CHANGE: f32 = PLAYER_FLIGHT_SPEED * 5.0;
const PLAYER_FLIGHT_MULT: f32 = 0.75;
const PLAYER_FLIGHT_STOP: f32 = PLAYER_FLIGHT_SPEED * 3.0;

const PLAYER_FLIGHT_TIME: f32 = 0.8;
const PLAYER_FLIGHT_TIME_SLOW: f32 = 0.25;

pub const PLAYER_GRAVITY: f32 = 128.0;
const PLAYER_FALL_GRAV_MULT: f32 = 3.0;
const PLAYER_FALL_SPEED:f32 = 128.0 * 1.2;
const PLAYER_JUMP_SPEED:f32 = 128.0 * 0.75;
const PLAYER_JUMP_CALCEL:f32 = 128.0 * 0.7/2.0;
const BOUNCE_SPEED: f32 = 64.0/2.0;

const PLAYER_SRC_SIZE:Vec2 = Vec2::splat(16.0);

const PLAYER_DEST_SIZE:Vec2 = Vec2::new(16.0, 16.0);
const PLAYER_SPRITE_OFFSET: Vec2 = Vec2::new(0.0, -2.0);

const PLAYER_TRAIL: Rect = Rect{x:128.0, y:0.0, w:8.0, h:8.0};
const PLAYER_TRAIL_SPEED:f32 = 24.0;

const COYOTE_TIME: f32 = 0.25;

const PLAYER_IDLE_ANIMATION: Animation = Animation::new(0, 3, 2.0, true);

const PLAYER_WALK_ANIMATION: Animation = Animation::new(5, 2, 0.5, true);

const PLAYER_JUMP_ANIMATION: Animation = Animation::new(3, 2, 0.4, true);

const PLAYER_FLY_ANIMATION: Animation = Animation::new(7, 1, 1.0, true);

enum PlayerState {
	Flying(Vec2), // (trail_position)
	Normal(f32), // (coyote_time)
}

pub struct Player {
	pub cbox: CollBox,
	motion: Vec2,
	grounded:bool,
	lm_dir: f32,

	anim_timer: Animation,
	player_state: PlayerState,


	flight_time: f32,
}

impl Player {
	pub const fn new() -> Self {
		Self {
			cbox: collbox!((0.0, 0.0), (4.0, 6.0)),// CollBox::new_f32(0.0, 0.0, 4.0, 6.0),
			motion: Vec2::ZERO,
			grounded: false,
			lm_dir: 1.0,
			anim_timer: PLAYER_IDLE_ANIMATION,
			player_state: PlayerState::Normal(0.0),
			flight_time: 0.0,
		}
	}

	pub fn update(&mut self, dt:f32) {
		let input = get_player_input();

		if input.dir.x != 0.0 {
			self.lm_dir = input.dir.x;
		}

		
		self.player_state = match self.player_state {
			PlayerState::Normal(coyote_time) => {
				self.normal_update(coyote_time,&input, dt)
			}

			PlayerState::Flying(trail_loc) => {
				self.flying_update(&input, trail_loc, dt)
			}

		};

		//self.anim_time =(self.anim_time + dt * if dir.x != 0.0 {MOVE_ANIMATION_SPEED} else {ANIMATION_SPEED}) % 3.0;
		self.anim_timer.update(dt);
	} // end update

	pub fn fly_box_check(&mut self, fb: &[CollBox]) {
		if self.cbox.check_intersection_slice(&fb) {
			self.set_to_flying();
		}
	}

	pub fn set_to_flying(&mut self) {
		self.flight_time = PLAYER_FLIGHT_TIME;
		self.player_state = PlayerState::Flying(
			match self.player_state {
				PlayerState::Flying(pos) => pos,
				
				_ => {
					self.motion.y *= PLAYER_FLIGHT_MULT;
					self.cbox.pos
				},
			}
		);
	}

	fn normal_update(&mut self, coyote_time: f32, input: &PlayerInput, dt: f32) -> PlayerState {
		self.motion.x = move_toward(self.motion.x, input.dir.x * PLAYER_SPEED, self.get_nspeed_change() * dt);
		self.motion.y += PLAYER_GRAVITY * dt * if self.motion.y > 0.0 {PLAYER_FALL_GRAV_MULT} else {1.0};
		self.motion.y = self.motion.y.min(PLAYER_FALL_SPEED);

		self.anim_timer.set_animation(
			match(input.dir.x.abs() as i32, self.grounded) {
				(1, true) => &PLAYER_WALK_ANIMATION,

				(0, true) => &PLAYER_IDLE_ANIMATION,

				(_, false) => &PLAYER_JUMP_ANIMATION,

				(_, _) => &PLAYER_IDLE_ANIMATION
			}
		);

		if input.jump && coyote_time > 0.0 {
			self.motion.y = -PLAYER_JUMP_SPEED;
		} else if !input.jump {
			self.motion.y = self.motion.y.max(-PLAYER_JUMP_CALCEL);
		}

		if input.should_fly() && self.flight_time > 0.0 {
			//self.motion = input.dir_norm * PLAYER_FLIGHT_SPEED;
			self.motion *= PLAYER_FLIGHT_MULT;
			return PlayerState::Flying(self.cbox.pos)
		}
		
		if self.grounded {
			self.flight_time = PLAYER_FLIGHT_TIME;
		}

		PlayerState::Normal(
			if self.grounded {COYOTE_TIME}
			else {
				if self.motion.y > 0.0 {f32::max(coyote_time - dt, 0.0)} else {0.0}})
	}

	fn flying_update(&mut self, input: &PlayerInput, trail_loc: Vec2, dt: f32) -> PlayerState {
		self.motion = move_toward_2D(self.motion, input.dir_norm * PLAYER_FLIGHT_SPEED, PLAYER_FLIGHT_CHANGE * dt);
		
		self.anim_timer.set_animation(&PLAYER_FLY_ANIMATION);
		
		let flight_drain_speed;
		if Vec2::dot(self.motion, input.dir_norm) <= 0.0 {
			self.motion = move_toward_2D(self.motion, Vec2::ZERO, PLAYER_FLIGHT_STOP * dt);
			flight_drain_speed = PLAYER_FLIGHT_TIME_SLOW;
		}
		else {
			flight_drain_speed = 1.0;
		}

		self.flight_time -= dt * flight_drain_speed;
		
		if self.flight_time < 0.0 || (is_key_down(KeyCode::Space) && self.flight_time < PLAYER_FLIGHT_TIME - 0.2) {
			if !self.grounded && self.motion.y <= 0.0 {
				self.motion.y = -BOUNCE_SPEED;
			}
			self.flight_time = 0.0;
			PlayerState::Normal(0.0)
		} else {
			PlayerState::Flying(move_toward_ex_2D(trail_loc, self.cbox.pos, PLAYER_TRAIL_SPEED*dt))
		}
	}

	fn get_nspeed_change(&self) -> f32 {
		(if self.grounded {
			PLAYER_SPEED_CHANGE
		}
		else {
			PLAYER_AIR_CHANGE
		}) * if self.motion.x * self.lm_dir <= 0.0 {
			PLAYER_REVERSE_CHANGE
		}else {
			1.0
		}
	}

	pub fn physics_update_mc (&mut self, walls: &[MultiCollider<CollBox>], breakable: &[CollBox], dt:f32) {
		self.cbox.pos.x += self.motion.x * dt;
		if MultiCollider::collide_x_slice(&mut self.cbox, &walls) {
			self.motion.x = 0.0;
		}

		if self.cbox.collide_x_slice(&breakable) {
			self.motion.x = 0.0;
		}
		
		self.cbox.pos.y += self.motion.y * dt;
		if self.grounded && self.motion.y >= 0.0 {self.cbox.pos.y += 0.01;}

		if MultiCollider::collide_y_slice(&mut self.cbox, &walls) || self.cbox.collide_y_slice(&breakable) {
			self.grounded = self.motion.y >= 0.0;
			self.motion.y = 0.0;
		}else {
			self.grounded = false;
		}
		
	}


	pub fn draw(&self, texture:&Texture2D) {
		let color = lerp_color(PLAYER_TIRED_COLOR, PLAYER_COLOR, self.flight_time/PLAYER_FLIGHT_TIME);
		let mut params;

		// draw tail
		if let PlayerState::Flying(trail_loc) = self.player_state {
			params = DrawTextureParams::default();
			params.source = Some(PLAYER_TRAIL);
			params.dest_size = Some(Vec2::splat(8.0));

			draw_centered_texture(&texture, trail_loc, color, params);

			//draw_texture_ex(&texture, draw_loc.x, draw_loc.y, color, params);
		}

		// draw body
		let draw_loc = self.cbox.pos + PLAYER_SPRITE_OFFSET - PLAYER_DEST_SIZE*0.5;
		params = DrawTextureParams::default();
		params.source = Some(Rect::new(0.01+PLAYER_SRC_SIZE.x * self.anim_timer.get_frame_idx() as f32, 0.01, PLAYER_SRC_SIZE.x-0.02, PLAYER_SRC_SIZE.y-0.02));
		params.dest_size = Some(PLAYER_DEST_SIZE);
		params.flip_x = self.lm_dir < 0.0;

		draw_centered_texture(&texture, self.cbox.pos+PLAYER_SPRITE_OFFSET, color, params);

		
	} // end draw
}

fn get_input_direction(neg: KeyCode, posit: KeyCode) -> f32 {
	let mut dir = 0.0;
	if is_key_down(neg) {
		dir -= 1.0;
	}

	if is_key_down(posit) {
		dir += 1.0;
	}
	dir
}

struct PlayerInput {
	jump: bool,
	fly: bool,
	dir: Vec2,
	dir_norm: Vec2
}

impl PlayerInput {
	pub fn should_fly(&self) -> bool {
		self.fly &&(!self.jump)
	}
}

fn get_player_input() -> PlayerInput {
	let mut dir_x = get_input_direction(KeyCode::A, KeyCode::D) + get_input_direction(KeyCode::Left, KeyCode::Right);
	dir_x = dir_x.clamp(-1.0, 1.0);

	let mut dir_y = get_input_direction(KeyCode::W, KeyCode::S) + get_input_direction(KeyCode::Up, KeyCode::Down);

	dir_y = dir_y.clamp(-1.0, 1.0);

	let dir = Vec2::new(dir_x, dir_y);

	PlayerInput{
		dir:dir,
		dir_norm: dir.normalize_or_zero(),
		jump: is_key_down(KeyCode::Space),
		fly: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::K),
	}
}