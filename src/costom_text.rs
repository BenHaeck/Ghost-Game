use macroquad::prelude::*;
use std::collections::{hash_map, HashMap};
pub struct RFont {
	texture: Texture2D,
	text_hash: HashMap<char, Rect>,
	def_rect: Rect,
	spacing: Vec2
}



impl RFont {
	pub fn new(texture: Texture2D, text_hash: HashMap<char, Rect>, def_rect: Rect, spacing: Vec2) -> Self{
		Self {
			texture: texture, text_hash: text_hash, def_rect: def_rect, spacing: spacing
		}
	}

	pub fn output_text(&self, text: &str) -> Vec<Rect> {
		let mut otp = Vec::with_capacity(text.len());
		for c in text.chars() {
			otp.push(match self.text_hash.get(&c) {
				Some(v) => v.clone(),
				None => self.def_rect
			});
		}

		otp
	}

	pub fn output_para(&self, text: &str) -> Vec<Vec<Rect>> {
		let iter = text.split("\n");
		let mut res = Vec::with_capacity(iter.clone().count());

		for s in iter {
			res.push(self.output_text(s));
		}

		res
	}

	pub fn draw_derived_text(&self, pos:Vec2, color: Color, font_size: f32, text: &[Rect]) {
		let mut delta = Vec2::ZERO;
		for i in 0..text.len() {
			let rect = &text[i];
			
			draw_texture_ex(&self.texture, pos.x + delta.x, pos.y + delta.y, color, DrawTextureParams{
				dest_size: Some(rect.size() * font_size),
				source: Some(*rect),
				..DrawTextureParams::default()
			});

			delta.x += font_size * self.spacing.x;
		}
	}

	pub fn calc_text_length(&self, font_size: f32, text:&[Rect]) -> f32 {
		let len = (text.len() as f32) * self.spacing.x;

		len * font_size
	}

	pub fn draw_derived_para(&self, pos:Vec2, color: Color, font_size: f32, para: &Vec<Vec<Rect>>) {
		for i in 0..para.len() {
			self.draw_derived_text(pos + vec2(0.0, font_size * self.spacing.y * (i as f32)), color, font_size, para[i].as_slice());
		}
	}
}

pub fn format_characters(hash_map:&mut HashMap<char,Rect>, characters: &str, offset: Vec2, char_size: Vec2) {
	let mut scan_pos = offset;
	

	for c in characters.chars() {
		hash_map.insert(c, Rect::new(scan_pos.x, scan_pos.y, char_size.x, char_size.y));
		scan_pos.x += char_size.x;
	}
}

pub fn format_alphabet (hash_map:&mut HashMap<char,Rect>, offset: Vec2, char_size: Vec2) {
	let mut scan_pos = offset;
	let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

	for c in alphabet.chars() {
		hash_map.insert(c, Rect::new(scan_pos.x, scan_pos.y, char_size.x, char_size.y));
		scan_pos.x += char_size.x;
	}

}