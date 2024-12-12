use macroquad::math::*;
use macroquad::color::*;

use crate::go_helpers::lerp_color;

pub const SCREEN_WIDTH: u32 = 16*4 * 7;
pub const SCREEN_HEIGHT: u32 = 16*4*4;

pub const LIGHT_DIV: u32 = 2;

pub const ZOOM:f32 = 4.0;



pub const SCREEN_DIM: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const CAM_WIDTH: f32 = SCREEN_WIDTH as f32 / ZOOM;
pub const CAM_HEIGHT: f32 = SCREEN_HEIGHT as f32 / ZOOM;
pub const CAM_DIM:Vec2 = vec2(CAM_WIDTH, CAM_HEIGHT);

pub const SHADOWMAP_DIM: UVec2 = uvec2(SCREEN_WIDTH / LIGHT_DIV, SCREEN_HEIGHT / LIGHT_DIV);
pub const SHADOW_CAM_DIM: Vec2 = vec2(SHADOWMAP_DIM.x as f32 / ZOOM, SHADOWMAP_DIM.y as f32 / ZOOM);

macro_rules! color_hex {
	($h:expr) => {
		{
			let v = ($h as u32).to_be_bytes();
			Color::new(v[0] as f32 / 255.0, v[1] as f32 / 255.0, v[2] as f32 / 255.0, v[3] as f32 / 255.0)
		}
	};
}

pub const BACKGROUND_COLOR: Color = color_hex!(0x464646ff);

pub const AMBIENT_LIGHT: Color = Color::new(0.1, 0.11, 0.12, 1.0);

pub const DANGER_COLOR: Color = color_hex!(0xc93523ff);
pub const DANGER_COLOR2: Color = color_hex!(0xee6c5dff);

pub const FB_COLOR: Color = color_hex!(0x17a24fff);

pub const DANGER_LIGHT: Color = color_hex!(0xf03924d4);

pub const PLAYER_COLOR: Color = color_hex!(0x88a1e2ff);
pub const PLAYER_TIRED_COLOR: Color = color_hex!(0x8d99a7ff);

pub const PLAYER_GLOW: Color = {
	let brightness = 0.5;
	Color::new(brightness*0.6, brightness*0.7, brightness, 1.0)
};

pub const EXIT_COLOR: Color = color_hex!(0xb69440ff);
pub const BREAKABLE_COLOR: Color = color_hex!(0x409075ff);
pub const BREAKABLE_COLOR_FADE: Color = color_hex!(0x28745aff);

pub const WINDOW_COLOR: Color = color_hex!(0x352f49ff);
pub const WINDOW_LIGHT: Color = color_hex!(0x7d89b5ff);