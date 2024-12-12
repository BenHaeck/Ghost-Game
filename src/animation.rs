/* things needed to animate an object
	frames
	playback speed
	current time
	length
	animation type(repeat or clamp)
*/



#[derive(Clone)]
pub struct Animation {
	time: f32,

	pub first: i32,
	pub num_frames:i32,
	pub length: f32,

	pub repeat: bool
}



impl Animation {
	pub const fn new(first:i32, num_frames:i32, length:f32, repeat: bool) -> Self {
		Self {first:first, num_frames:num_frames, time: 0.0, length: length, repeat: repeat }
	}

	pub fn update(&mut self, dt: f32) {
		self.set_time(self.time + dt);
		
	}

	pub fn set_time(&mut self, new_time: f32) {
		
		self.time = if self.repeat {
			if new_time < 0.0 {
				self.length - new_time % self.length
			} else {
				new_time % self.length
			}
		} else {
			new_time.max(0.0).min(self.length)
		}
	}

	pub const fn get_time(&self) -> f32 {self.time}

	pub fn get_frame(&self) -> i32{
		((self.time / self.length) * self.num_frames as f32) as i32 
	}

	pub fn get_frame_idx(&self) -> i32 {
		self.first + self.get_frame()
	}

	pub fn set_animation(&mut self, anim: &Animation) {
		*self = Self{
			time: 
			if self.num_frames == anim.num_frames && self.length == anim.length {
				self.time
			}else {
				0.0
			},
			..*anim
		}
	}
}

