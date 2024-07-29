use macroquad::math::*;


macro_rules! collbox {
	(($x:expr, $y:expr), ($w:expr, $h:expr)) => {
		collbox!(vec2($x, $y), vec2($w, $h))
	};

	($pos:expr, ($w:expr, $h:expr)) => {
		collbox!($pos, vec2($w, $h))
	};

	(($x:expr, $y:expr), $dim:expr) => {
		collbox!(vec2($x, $y), $dim)
	};

	($pos:expr, $half_dim:expr) => {
		CollBox::new($pos, $half_dim)
	};

}

pub(crate) use collbox;

macro_rules! collcircle {
	(($x:expr, $y:expr), $r: expr) => {
		collcircle!(vec2($x, $y), $r)
	};

	($pos: expr, $r: expr) => {
		CollCircle::new($pos, $r)
	};
}

pub(crate) use collcircle;

pub trait Collider {
	fn check_intersection (&self, other: &Self) -> bool;

	fn check_intersection_slice (&self, others: &[Self]) -> bool where Self:Sized {
		for i in 0..others.len() {
			if self.check_intersection(&others[i]) {
				return true;
			}
		}

		false
	}
}

#[derive(Clone, Copy)]
pub struct CollBox {
	pub pos: Vec2,
	pub half_dim: Vec2
}

impl Collider for CollBox {
	

	fn check_intersection(&self, other: &Self) -> bool {
		let dist =(self.pos - other.pos).abs();
		let comb_dim = self.half_dim + other.half_dim;

		dist.x < comb_dim.x && dist.y < comb_dim.y
	}
}

impl CollBox {
	pub const fn new(pos: Vec2, half_dim: Vec2) -> Self {
		Self { pos: pos, half_dim: half_dim }
	}

	pub const fn new_f32(pos_x: f32, pos_y: f32, half_width: f32, half_height: f32) -> Self {
		Self {
			pos: Vec2::new(pos_x, pos_y),
			half_dim: Vec2::new(half_width, half_height)
		}
	}

	pub fn align_edge_x(&mut self, other: &Self) {
		self.pos.x = other.pos.x -(self.half_dim.x + other.half_dim.x) * f32::signum(other.pos.x - self.pos.x);
	}
	pub fn align_edge_y(&mut self, other: &Self) {
		self.pos.y = other.pos.y -(self.half_dim.y + other.half_dim.y) * f32::signum(other.pos.y - self.pos.y);
	}

	pub fn collide_x(&mut self, other: &Self) -> bool {
		if (self.check_intersection(&other)) {
			self.align_edge_x(&other);
			true
		} else{
			false
		}
	}
	pub fn collide_y(&mut self, other: &Self) -> bool {
		if (self.check_intersection(&other)) {
			self.align_edge_y(&other);
			true
		} else{
			false
		}
	}

	pub fn collide_x_slice(&mut self, other: &[Self]) -> bool {
		let mut res = false;
		for i in 0..other.len() {
			res = self.collide_x(&other[i]) || res;
		}
		res
	}
	pub fn collide_y_slice(&mut self, other: &[Self]) -> bool {
		let mut res = false;
		for i in 0..other.len() {
			res = self.collide_y(&other[i]) || res;
		}
		res
	}

	pub fn to_circle(&self) -> CollCircle {
		CollCircle::new(self.pos, (self.half_dim.x + self.half_dim.y) * 0.5)
	}
}

pub struct CollCircle {
	pub pos: Vec2,
	pub radius: f32,
}

impl Copy for CollCircle {}

impl Clone for CollCircle {
	fn clone(&self) -> Self {
		return Self{..*self};
	}
}

impl Collider for CollCircle {
	fn check_intersection (&self, other: &Self) -> bool {
		let dif = self.pos - other.pos;
		let comb_radius = self.radius + other.radius;

		dif.length() < comb_radius
	}
}

impl CollCircle {
	pub fn new (pos: Vec2, radius: f32) -> Self {
		Self{pos:pos, radius: radius}
	}

	pub fn to_box(&self) -> CollBox{
		CollBox::new(self.pos, Vec2::splat(self.radius))
	}
}

pub fn check_circle_box_intersection (circle: &CollCircle, cbox: &CollBox) -> bool {
	let nearest_point = circle.pos.clamp(cbox.pos - cbox.half_dim, cbox.pos + cbox.half_dim);
	circle.check_intersection(&collcircle!(nearest_point, 0.0))
}

pub struct MultiCollider <TC> where TC: Collider + Clone {
	pub main: TC,
	multi: Vec<TC>
}

#[allow(dead_code)]
impl <TC> MultiCollider<TC> where TC: Collider+Clone {
	pub fn new (filter_coll: TC) -> Self {
		Self{
			main:filter_coll,
			multi: Vec::new()
		}
	}

	pub fn check_intersection (&self, other: &TC) -> bool {
		if !other.check_intersection(&self.main) {
			return false;
		}

		for i in 0..self.multi.len() {
			if other.check_intersection(&self.multi[i]) {
				return true;
			}
		}

		false
	}

	pub fn check_intersection_slice(coll: &[Self], other:&TC) -> bool {
		for i in 0..coll.len() {
			if coll[i].check_intersection(&other) {
				return true;
			}
		}
		
		false
	}

	pub fn add_if_intersect (&mut self, coll: &TC) {
		if self.main.check_intersection(&coll) {
			self.multi.push(coll.clone());
		}
	}

	pub fn read_multi<'a>(&'a self) -> &'a Vec<TC> {
		return &self.multi;
	}

	pub fn clear(&mut self) {
		self.multi.clear();
	}

	pub fn add_if_intersect_slice (mc: &mut [Self], coll:&TC) {
		for i in 0..mc.len() {
			mc[i].add_if_intersect(&coll);
		}
	}

	pub fn call_on_intersect<F>(&self, other: &TC, mut f: F) where F:FnMut(&TC) {
		if !self.main.check_intersection(&other) {return;}

		for i in 0..self.multi.len() {
			f(&self.multi[i]);
		}
	}
}

#[allow(dead_code)]
impl MultiCollider<CollBox> {
	pub fn collide_x (&self, other:&mut CollBox) -> bool {
		if !other.check_intersection(&self.main) {
			return false;
		}
		let mut result = false;
		for i in 0..self.multi.len() {
			if other.check_intersection(&self.multi[i]) {
				other.align_edge_x(&self.multi[i]);
				result = true;
			}
		}

		result
	}

	pub fn collide_x_slice(coll: &mut CollBox, ss: &[Self])-> bool {
		let mut result = false;
		for i in 0..ss.len() {
			result = result|| ss[i].collide_x(&mut *coll);
		}

		result
	}

	pub fn collide_y (&self, other:&mut CollBox) -> bool {
		if !other.check_intersection(&self.main) {
			return false;
		}

		let mut result = false;
		for i in 0..self.multi.len() {
			if other.check_intersection(&self.multi[i]) {
				other.align_edge_y(&self.multi[i]);
				result = true
			}
		}

		result
	}

	pub fn collide_y_slice(coll: &mut CollBox, ss: &[Self])-> bool {
		let mut result = false;
		//let mut in_sectors = 0;
		//let mut checked = 0;
		for i in 0..ss.len() {
			result = result|| ss[i].collide_y(&mut *coll);
			/*if ss[i].main.check_intersection(&coll) {
				in_sectors+=1;
				checked+=ss[i].read_multi().len();
			}*/
		}
		//println!("in {in_sectors} checked {checked}");

		result
	}
}


#[allow(dead_code)]
pub fn generate_multi_col_grid(offset:Vec2, size:Vec2, num_elements:UVec2) -> Vec<MultiCollider<CollBox>> {
	let mut result = Vec::with_capacity((num_elements.x * num_elements.y) as usize);
	for x in 0..num_elements.x {
		for y in 0..num_elements.y {
			let pos = offset + vec2(size.x * x as f32, size.y * y as f32) + size * 0.5;
			result.push(MultiCollider::new(CollBox::new(pos, size * 0.5)));
		}
	}

	result
}