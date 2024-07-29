use macroquad::math::*;
use macroquad::color::*;

pub const SCREEN_WIDTH: u32 = 16*4 * 7;
pub const SCREEN_HEIGHT: u32 = 16*4*4;

pub const ZOOM:f32 = 4.0;



pub const SCREEN_DIM: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const CAM_WIDTH: u32 = SCREEN_WIDTH / ZOOM as u32;
pub const CAM_HEIGHT: u32 = SCREEN_HEIGHT / ZOOM as u32;
pub const CAM_DIM:UVec2 = uvec2(CAM_WIDTH, CAM_HEIGHT);

macro_rules! make_color {
	($r:expr, $g: expr, $b:expr) => {
		Color::new($r * 0.01, $g*0.01, $b*0.01, 1.0)
	}
}

macro_rules! color_hex {
	($h:expr) => {
		{
			let v = ($h as u32).to_be_bytes();
			Color::new(v[0] as f32 / 255.0, v[1] as f32 / 255.0, v[2] as f32 / 255.0, v[3] as f32 / 255.0)
		}
	};
}

pub const DANGER_COLOR: Color = color_hex!(0xc93523ff);

pub const PLAYER_COLOR: Color = color_hex!(0x88a1e2ff);
pub const PLAYER_TIRED_COLOR: Color = color_hex!(0x94949dff);

pub const EXIT_COLOR: Color = color_hex!(0xb69440ff);
pub const BREAKABLE_COLOR: Color = color_hex!(0x409075ff);