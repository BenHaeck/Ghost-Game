use macroquad::color;
use macroquad::prelude::*;
use macroquad::rand::*;

use crate::options::*;

use crate::go_helpers::*;

const COLOR_FADES: &[Color] = &[WHITE, WHITE, DANGER_COLOR, DANGER_COLOR2, BREAKABLE_COLOR, BREAKABLE_COLOR_FADE, PLAYER_COLOR, PLAYER_TIRED_COLOR];

#[derive(Clone, Copy)]
pub enum ParticalRenderer{
	Circle = 0,
	Dust = 1
}

pub struct ParticalSystem {
	particals: Vec<Partical>,
}

impl ParticalSystem {
	pub fn new () -> Self{
		Self { particals: Vec::new() }
	}

	

	pub fn create_partical(&mut self, num: u32, from: Vec2, from_rad: f32, to:Vec2, to_rad:f32, lifetime: f32, color_fade: u8, renderer: ParticalRenderer) {
		for _i in 0..num{
			let point_distance = gen_range(0.0, 1.0);
			let mut point = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0));
			point = vec2(point.x * point.x.abs(), point.y * point.y.abs());
			point = point.normalize_or_zero() * (0.5 + point_distance * point_distance*0.5);

			self.particals.push(Partical {
				pos: point * from_rad + from,
				new_pos: point * to_rad + to,
				time: 0.0,
				life_time: lifetime,
				color_fade,
				renderer: renderer
			});
		}
	}


	pub fn update(&mut self, dt: f32) {
		for i in (0..self.particals.len()).rev() {
			self.particals[i].time += dt;
			if self.particals[i].time >= self.particals[i].life_time {
				self.particals.remove(i);
			}
		}
	}

	pub fn draw(&self, texture: &Texture2D) {
		for i in 0..self.particals.len() {
			let partical = &self.particals[i];
			let time = partical.time / partical.life_time;
			let mut adjusted_time = 1.0-time;
			adjusted_time = (adjusted_time+adjusted_time*adjusted_time)*0.5;
			adjusted_time = 1.0-adjusted_time;
			let pos = Vec2::lerp(partical.pos, partical.new_pos, adjusted_time);

			let anim_frame = (time.min(1.0)*4.0).floor();

			let color_idx = partical.color_fade * 2;
			let color = lerp_color(COLOR_FADES[color_idx as usize], COLOR_FADES[color_idx as usize+1], adjusted_time);

			draw_centered_texture(texture, pos, color, DrawTextureParams{
				dest_size: Some(Vec2::splat(8.0)),
				source: Some(Rect::new(anim_frame*8.0, 8.0*(partical.renderer as i32 as f32), 8.0, 8.0)),
				..Default::default()
			})
		}

	}
	pub fn get_count(&self) -> usize {
		self.particals.len()
	}
}

pub struct Partical {
	pub pos: Vec2,
	pub new_pos: Vec2,
	pub time: f32,
	pub life_time: f32,
	pub color_fade: u8,
	pub renderer: ParticalRenderer,
	
}