use crate::collision::*;
use crate::wall_man::*;

use crate::options::*;

use crate::go_helpers::*;

use macroquad::prelude::*;



pub fn assign_light_type (light_pos: Vec2, world_man: &[MultiCollider<CollBox>]) -> char {
	if MultiCollider::check_intersection_slice(&world_man, &collbox!(light_pos + vec2(0.0, -16.0), Vec2::ZERO)) {
		'c'
	} else {
		'w'
	}
}

pub fn draw_light_sources(lights: &[(Vec2, char)], tex: &Texture2D) {
	for i in 0..lights.len() {
		let (light_pos, light_type) = lights[i];
		match light_type {
			'c' => draw_centered_texture(&tex, light_pos, EXIT_COLOR, DrawTextureParams{
				dest_size: Some(vec2(16.0, 16.0)),
				source: Some(Rect::new(96.0, 80.0, 16.0, 16.0)),
				..Default::default()
			}),

			_ => draw_centered_texture(&tex, light_pos, WINDOW_COLOR, DrawTextureParams{
				dest_size: Some(vec2(16.0, 16.0)),
				source: Some(Rect::new(96.0, 64.0, 16.0, 16.0)),
				..Default::default()
			}),
		}
	}
}

pub fn draw_lights(lights: &[(Vec2, char)], tex:&Texture2D) {
	for i in 0..lights.len() {
		let (light_pos, light_type) = lights[i];
		match light_type {
			'c' => {
				let light_size = vec2(64.0+32.0, 64.0+48.0);
				draw_centered_texture(&tex, light_pos + vec2(0.0, light_size.y * 0.5-12.0), WHITE, DrawTextureParams{
					dest_size: Some(light_size),
					source: Some(Rect::new(33.0, 0.0, 32.0, 32.0)),
					..Default::default()
				})
			}

			_ => draw_centered_texture(&tex, light_pos, WINDOW_LIGHT, DrawTextureParams{
				dest_size: Some(Vec2::splat(128.0)),
				source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
				..Default::default()
			}),
		}
	}
}