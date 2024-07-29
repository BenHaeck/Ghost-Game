use macroquad::prelude::*;

use crate::collision::*;


pub fn move_toward(val: f32, target:f32, delta:f32) -> f32 {
	let vt_dif = target - val;

	if vt_dif.abs() > delta {
		val + vt_dif.signum() * delta
	} else {
		target
	}
}

#[allow(non_snake_case)]
pub fn move_toward_2D(val: Vec2, target:Vec2, delta:f32) -> Vec2 {
	if delta <= 0.0 {return val;}
	let vt_dif = target - val;

	if vt_dif.length() > delta {
		val + vt_dif.normalize() * delta
	} else {
		target
	}
}

#[allow(non_snake_case)]
pub fn move_toward_ex_2D(val: Vec2, target:Vec2, delta:f32) -> Vec2{
	// (v-t) * e + t
	// v*e - t*e + t
	// v*e - t*(1-e)
	//(val - target) * f32::exp(-delta) + target
	let e = (-delta).exp();
	val * e + (1.0-e)*target
}

pub fn lerp_color (color1: Color, color2: Color, s: f32) -> Color {
	Color::from_vec( Vec4::lerp(color1.to_vec(), color2.to_vec(), s))
}

/*pub struct GOContainer<TShared, TUnique>{
	pub shared: TShared,
	pub unique: TUnique,
}*/

pub fn run_and_remove<T, F>(vals: &mut Vec<T>, mut f: F) where F: FnMut (&mut T) -> bool {
	for i in (0..vals.len()).rev() {
		if f(&mut vals[i]) {
			vals.remove(i);
		}
	}
}

pub fn list_interact<T1, T2, F> (vals1: &mut [T1], vals2: &mut [T2], f: F) where F: Fn (&mut T1, &mut T2) {
	for i in 0..vals1.len() {
		for j in 0..vals2.len() {
			f(&mut vals1[i], &mut vals2[j]);
		}
	}
}

pub fn query<T, F> (vals: &[T], f: F) -> bool where F: Fn(&T) -> bool {
	for i in 0..vals.len() {
		if f(&vals[i]) {return true;}
	}
	false
}

pub trait Entity <TShared> {
	fn update(&mut self, input: &mut TShared, dt: f32);

	fn update_all (entities: &mut [Self], input: &mut TShared, dt:f32) where Self:Sized {
		for i in 0..entities.len() {
			entities[i].update(&mut *input, dt);
		}
	}

	fn should_remove (&self) -> bool;

	fn clean(entities: &mut Vec<Self>) where Self: Sized {
		for i in 0..entities.len() {
			if entities[i].should_remove() {
				entities.remove(i);
			}
		}
	}
}

pub fn draw_coll_box (b: CollBox, color: Color) {
	let draw_loc = b.pos -  b.half_dim;
	draw_rectangle(draw_loc.x, draw_loc.y, b.half_dim.x * 2.0, b.half_dim.y * 2.0, color);
}

pub fn draw_centered_texture(texture: &Texture2D, pos: Vec2, color: Color, params: DrawTextureParams) {
	let draw_loc = pos - match params.dest_size {
		Some(s) => s,

		None => texture.size(),
	} * 0.5;
	draw_texture_ex(texture, draw_loc.x, draw_loc.y, color, params);
}