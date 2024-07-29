
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default;

use macroquad::prelude::*;
use crate::collision::*;
use crate::enemy::*;
use crate::go_helpers::*;

use crate::parser::*;

use crate::player;
use crate::player::*;
use crate::staticobj::*;
use crate::wall_man::*;

use crate::options::*;
use crate::costom_text::*;

const DEF_TILE:Vec2 = vec2(0.0, 3.0);

const TILE_SIZE: f32 = 16.0;
const TILE_VEC: Vec2 = Vec2::splat(TILE_SIZE);

const CAM_SPEED: f32 = 8.0;


pub struct GameWorld {
	assets: AssetManager,

	// gameobjects
	triggers: Vec<Trigger>,

	breakable_walls: Vec<CollBox>,
	
	enemies: Vec<Enemy>,

	bullets: Vec<Bullet>,

	pub player: Player,

	pub cam_position: Vec2,

	world_size: Vec2,

	level_exit: Option<(CollBox, f32)>,

	note: Option<Vec2>,

	// level information
	time_speed: f32,
	tileset: u32,
	
	pub level_index: i32,
	pub load_new_level: bool,
	level_blueprint: Vec<LevTile>,


	hud: HUD,
	//wall_stuff: Vec<MultiCollider<CollBox>>
	wall_man: WallMan<Vec2>,
}

impl GameWorld {
	pub fn new(assets: AssetManager) -> Self {
		let gw = Self {
			triggers: Vec::new(),
			breakable_walls: Vec::new(),

			enemies: Vec::new(),

			bullets: Vec::new(),
			
			player: Player::new(),
			level_exit: Option::None,

			note: Option::None,

			cam_position: Vec2::ZERO,

			world_size: CAM_DIM.as_vec2(),

			time_speed: 1.0,
			tileset: 0,
			level_index: 0,
			load_new_level: false,
			level_blueprint: Vec::new(),


			hud: HUD::new(&assets, ""),
			assets: assets,


			//wall_stuff: generate_multi_col_grid(Vec2::new(0.0, 16.0), Vec2::splat(128.0), UVec2::new(4, 3)),

			wall_man: WallMan::new(generate_multi_col_grid(Vec2::ZERO, Vec2::splat(128.0), uvec2(5,3)), DEF_TILE),
		};
		gw
	}

	pub fn setup(&mut self) {
		// resets the game world
		self.player = Player::new();

		self.triggers.clear();
		self.breakable_walls.clear();

		self.enemies.clear();
		self.bullets.clear();
		self.level_exit = None;
		self.note = None;

		let wm_grid_size = 64.0;
		self.wall_man = WallMan::new(generate_multi_col_grid(Vec2::splat(-8.0),
			Vec2::splat(wm_grid_size),
			UVec2::ONE+(self.world_size/wm_grid_size).as_uvec2()
		),DEF_TILE);
		
		//self.world_size = CAM_DIM.as_vec2();

		// parses the layout
		for bp_tile in self.level_blueprint.as_slice() {
			let ent_pos = bp_tile.pos;
			//self.world_size.x = self.world_size.x.max(ent_pos.x);
			//self.world_size.y = self.world_size.y.max(ent_pos.y);
			match bp_tile.ty {
				'p' => 	self.player.cbox.pos = ent_pos, // player

				'#' => self.wall_man.add_wall(collbox!(ent_pos, TILE_VEC*0.5)),// wall

				'E' => self.enemies.push(Enemy::new(ent_pos, EnemyType::AngryGhosts(false))), // enemies

				'T' => self.enemies.push(new_turret(ent_pos)),

				'^' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(7.0)), TriggerType::Trap(0.0))),

				'f' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(46.0*0.5)), TriggerType::FlyBox)), // fly_boxes
				
				'*' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(7.0)), TriggerType::Spikes)), // death_box

				'/' => self.breakable_walls.push(collbox!(ent_pos, TILE_VEC*0.5)),

				'X' => self.level_exit = Some((collbox!(ent_pos, (5.0, 8.0)), 0.0)), // exit

				'N' => self.note = Some(ent_pos), // Notes

				_ => {}
			}
		}

		self.cam_position = self.player.cbox.pos;
		self.constrain_cam();

		self.triggers.sort_by(|a, b|{
			let av = match a.trigger_type {
				TriggerType::Spikes => 0,
				TriggerType::FlyBox => 10,
				_ => 9,
			};

			let bv = match b.trigger_type {
				TriggerType::Spikes => 0,
				TriggerType::FlyBox => 10,
				_ => 9,
			};

			if av == bv {
				Ordering::Equal
			} else if av > bv {
				Ordering::Less
			}else {Ordering::Greater}
		},);

		self.wall_man.cull();

		self.wall_man.assign_tile_types(|exp, coll| {
			//return vec2(0.0, 3.0);
			match (coll.x, coll.y) {
				(true, true) => (ivec2(1, 1) - exp).as_vec2(),
				(true, false) => if exp.x == 0 {vec2(2.0, 3.0)} else {DEF_TILE},
				(false, true) => if exp.y == 0 {vec2(1.0, 3.0)} else {DEF_TILE},
				(_) => vec2(0.0, 3.0),
			}
		}, Vec2::splat(8.5), Vec2::splat(0.1));
	}

	
	pub fn update(&mut self, dt:f32) {
		macro_rules! safe_reset{
			() => {
				self.setup();
				return;
			}
		}

		// player logic
		let mut queue_reset = false;
		self.player.update(dt * self.time_speed);
		let mut player_in_trap = false;
		self.player.physics_update_mc(&self.wall_man.wall_colliders.as_slice(), &self.breakable_walls, dt * self.time_speed);
		Trigger::coll_check(self.triggers.as_mut_slice(), &self.player.cbox.clone(), |trigger|{
			match trigger.trigger_type {
				TriggerType::FlyBox => self.player.set_to_flying(),
				
				TriggerType::Spikes => queue_reset = true,

				TriggerType::Trap(_) => player_in_trap = true,
				_ => {}
			}
			true
		});

		if self.player.cbox.pos.y > self.world_size.y || query(self.enemies.as_slice(), |enemy| {enemy.cbox.check_intersection(&self.player.cbox)}){
			safe_reset!();
		}

		self.player.cbox.pos.y = 0.0f32.max(self.player.cbox.pos.y);

		// camera logic
		self.cam_position = move_toward_ex_2D(self.cam_position, self.player.cbox.pos, dt*CAM_SPEED);
		self.constrain_cam();
		
		// enemy logic
		run_and_remove(&mut self.enemies, |enemy| {
			enemy.update(&self.player.cbox, &mut self.bullets, player_in_trap, dt * self.time_speed);
			let mut dead = false;
			for i in &self.triggers {
				match i.trigger_type {
					TriggerType::Spikes => {
						if i.coll.check_intersection(&enemy.cbox) {
							dead = true;
							break;
						}
					}

					_ => {}
				}
			}
			return dead;
		});

		// bullet logic
		run_and_remove(&mut self.bullets, |bullet|{
			bullet.update(dt);
			if query(&self.wall_man.wall_colliders.as_slice(), |mc| {
				mc.check_intersection(&bullet.coll.to_box())
			}) {
				return true;
			}
			bullet.life_time_update(dt)
		});

		// enemy bullet interactions
		for e in (0..self.enemies.len()).rev() {
			if !self.enemies[e].get_friendly_fire() {continue;}
			let enemy = &self.enemies[e].cbox;
			let mut remove_enemy = false;
			for b in (0..self.bullets.len()).rev() {
				let bullet = &self.bullets[b].coll;
				if check_circle_box_intersection(&bullet, &enemy) {
					remove_enemy = true;
					self.bullets.remove(b);
				}
			}
			if remove_enemy {
				self.enemies.remove(e);
			}
		}
		
		run_and_remove(&mut self.breakable_walls, |br| {
			query(&self.bullets, |bu| {check_circle_box_intersection(&bu.coll, &br)})
		});

		if query(self.bullets.as_slice(), |bullet|
			{check_circle_box_intersection(&bullet.coll, &self.player.cbox)}
		) {
			safe_reset!();
		}

		// door update
		if let Some(exit) = self.level_exit {
			let mut n_velocity_y = exit.1 + dt * PLAYER_GRAVITY;
			let mut nbox = exit.0;
			nbox.pos.y += n_velocity_y * dt;
			if MultiCollider::collide_y_slice(&mut nbox, &self.wall_man.wall_colliders)
			|| nbox.collide_y_slice(&self.breakable_walls){
				n_velocity_y = 0.0;
			}

			self.level_exit = Some((nbox, n_velocity_y));
		}

		// trigger update
		Trigger::update(self.triggers.as_mut_slice(), dt);

		self.hud.update(
			match self.note {
				Some(note) => {
					self.player.cbox.check_intersection(&collbox!(note, (24.0, 32.0)))
				}
				None => {false}
			}, dt);

		if queue_reset {
			safe_reset!();
		}
	}

	

	pub fn draw(&self) {
		draw_background(&self.assets.world_images, Some(Rect::new(16.0 + 48.0 * self.tileset as f32, 16.0, 16.0, 16.0)), self.cam_position);

		match &self.level_exit {
			Some(exit) => {
				draw_centered_texture(&self.assets.world_images, exit.0.pos, EXIT_COLOR, DrawTextureParams{
					dest_size: Some(TILE_VEC),
					source: Some(Rect::new(0.0, 112.0, 16.0, 16.0)),
					..DrawTextureParams::default()
				});
			}

			None => {}
		}

		match &self.note {
			Some(note) => {
				draw_centered_texture(&self.assets.world_images, note.clone(), WHITE, DrawTextureParams{
					source: Some(Rect::new(96.0, 96.0, 16.0, 16.0)),
					dest_size: Some(TILE_VEC),

					..Default::default()
				})
			}

			None => {}
		}

		for i in 0..self.triggers.len() {
			self.triggers[i].draw(&self.assets.world_images);
		}

		self.player.draw(&self.assets.player_image);

		self.wall_man.draw_walls(|wall_pos, wall_tile| {
			let wall_size = TILE_VEC;
			let draw_loc = wall_pos - wall_size * 0.5;

			let wall_src_pos = (wall_tile.clone()+Vec2::new(3.0 * (self.tileset as f32), 0.0)) * Vec2::splat(16.0);
			//println!("{}", &wall_tile);
			draw_centered_texture(&self.assets.world_images, wall_pos, WHITE, DrawTextureParams{
				source: Some(Rect::new(wall_src_pos.x, wall_src_pos.y, 16.0, 16.0)),
				dest_size: Some(wall_size),
				..DrawTextureParams::default()
			});
		});

		for i in 0..self.breakable_walls.len() {
			draw_centered_texture(&self.assets.world_images, self.breakable_walls[i].pos, BREAKABLE_COLOR, DrawTextureParams{
				dest_size: Some(Vec2::splat(16.0)),
				source: Some(Rect::new(96.0, 112.0, 16.0, 16.0)),
				..Default::default()
			})
		}

		

		for i in 0..self.enemies.len() {
			let enemy = &self.enemies[i];
			enemy.draw(&self.assets.dangers);
		}

		for i in 0..self.bullets.len(){
			let bullet = &self.bullets[i];
			
			draw_centered_texture(&self.assets.dangers, bullet.coll.pos, DANGER_COLOR, DrawTextureParams{
				dest_size: Some(TILE_VEC),
				source: Some(Rect::new(0.0, 16.0, 16.0, 16.0)),
				pivot: Some(bullet.coll.pos),
				rotation: f32::atan2(bullet.motion.y, bullet.motion.x),
				..DrawTextureParams::default()
			})
		}
	}

	pub fn draw_hud (&self) {
		let top_right = self.cam_position + vec2(CAM_WIDTH as f32, -(CAM_HEIGHT as f32));
		let font = &self.assets.font;

		self.hud.draw(&self.assets, self.cam_position);
	}

	pub async fn handle_levels(&mut self, level_file_names: &Vec<String>) {
		macro_rules! load_next_level {
			() => {
				self.level_index = (self.level_index+1) % (level_file_names.len() as i32);
				self.load_level(format!("Levels/{}.par", level_file_names[self.level_index as usize]).as_str()).await;
				self.setup();
				return;
			}
		}
		match &self.level_exit {
			Some(exit) => {
				if self.player.cbox.check_intersection(&exit.0) {
					load_next_level!();
				}
			}

			None => {}
		}
	}

	pub async fn load_level(&mut self, level_path:&str) {
		match load_string(level_path).await {
			Result::Ok(level_file)=> {
				let parser = StringParser::new(level_file.as_str());
				self.time_speed = parser.get_float_or_def("timeScale", 1.0);
				
				let mut level_string = parser.get_as_string_literal_or_def("layout", "P\n\n#");
				level_string = level_string.replace("\n\n", "\n");
				//parser.print_names();
				level_string.remove(0);
				self.tileset = parser.get_int_or_def("tileset", 0).abs() as u32;

				self.hud = HUD::new(&self.assets, &parser.get_as_string_literal_or_def("noteText", "").to_uppercase());

				self.level_blueprint = read_level(&level_string, TILE_SIZE);
				self.world_size = get_level_size(&level_string).as_vec2() * TILE_SIZE;
				self.world_size -= Vec2::ONE*TILE_SIZE;
			}

			_ => {
				println!("Error. File {} probably does not exist, or something", &level_path);
			}
		}
	}

	pub fn constrain_cam(&mut self) {
		self.cam_position = self.cam_position.clamp(CAM_DIM.as_vec2(), self.world_size - CAM_DIM.as_vec2());
	}

	pub fn set_assets(&mut self, assets:AssetManager) {
		self.assets = assets;
	}
}

pub struct AssetManager {
	player_image: Texture2D,
	dangers: Texture2D,
	world_images: Texture2D,
	font: RFont
}

impl AssetManager {
	pub async fn new() -> Self {
		let player_image: Texture2D = load_texture("Images/Player.png").await.unwrap();
		player_image.set_filter(FilterMode::Nearest);

		let world_images = load_texture("Images/LevelImages.png").await.unwrap();
		world_images.set_filter(FilterMode::Nearest);

		let dangers = load_texture("Images/Dangers.png").await.unwrap();
		dangers.set_filter(FilterMode::Nearest);
		
		let font_image = load_texture("Images/Font.png").await.unwrap();
		font_image.set_filter(FilterMode::Nearest);
		let mut hash = HashMap::new();

		format_alphabet(&mut hash, Vec2::ZERO, Vec2::splat(8.0));

		format_characters(&mut hash, "0123456789.,:!", vec2(0.0, 8.0), Vec2::splat(8.0));
		
		Self {
			player_image: player_image,
			world_images: world_images,
			dangers: dangers,

			font: RFont::new(font_image, hash, Rect::new(0.0, 0.0, 1.0, 1.0), vec2(7.0, 9.0)),
		}
	}
}

pub struct HUD {
	note_text: Vec<Vec<Rect>>,

	note_opacity: f32,
}

impl HUD {
	pub fn new(assets: &AssetManager, note_text: &str) -> Self {
		Self {
			note_text: assets.font.output_para(note_text),

			note_opacity: 0.0
		}
	}

	pub fn update (&mut self, player_reading: bool, dt: f32) {
		self.note_opacity = (self.note_opacity + dt * 
			if player_reading {1.0} else {-1.0}
		).clamp(0.0, 1.0);
	}

	pub fn draw (&self, assets: &AssetManager, camera_pos: Vec2) {
		let offset = vec2(-64.0, 0.0);
		let mut color = WHITE;
		color.a = self.note_opacity;
		assets.font.draw_derived_para(camera_pos + offset, color, 0.5, &self.note_text);
	}
}

pub struct LevTile {
	pub pos: Vec2,
	pub ty: char,
}



fn read_level(layout: &str, grid_size: f32) -> Vec<LevTile> {
	let mut tiles = Vec::new();
	let mut pos = Vec2::ZERO;
	for c in layout.chars() {
		match c {
			' ' => {
				pos.x += grid_size;
			}

			'\n' | '\r' => {
				pos.x = 0.0;
				pos.y += grid_size;
			}
			
			_ => {
				tiles.push(LevTile{pos:pos, ty:c.clone()});
				//println!("{} -> {}", pos, c);
				pos.x += grid_size;
			}
		}
	}

	tiles
}

fn draw_background (texture: &Texture2D, rect: Option<Rect>, camera_pos: Vec2) {
	let offset = vec2(4.0, -4.0);
	let use_pos = ((camera_pos+offset) / TILE_SIZE).round() * TILE_SIZE -offset;

	let starting = use_pos - CAM_DIM.as_vec2() - Vec2::splat(TILE_SIZE);
	let ending = use_pos + CAM_DIM.as_vec2() + Vec2::splat(TILE_SIZE);

	let mut x = starting.x;

	while x < ending.x {
		let mut y = starting.y;
		while y < ending.y {
			draw_centered_texture(&texture, vec2(x, y), DARKGRAY, DrawTextureParams{
				source: rect,
				dest_size: Some(Vec2::splat(TILE_SIZE)),
				..Default::default()
			});
			y += TILE_SIZE;
		}
		x += TILE_SIZE;
	}

}

fn get_level_size(layout: &str) -> UVec2 {
	let mut res = UVec2::ZERO;
	let iter = layout.split('\n');
	res.y = iter.clone().count() as u32;

	for s in iter {
		res.x = res.x.max(s.len() as u32);
	}

	res
}

fn draw_coll_rect(rect: &CollBox, color: Color) {
	let draw_loc = rect.pos - rect.half_dim;
	draw_rectangle(draw_loc.x, draw_loc.y, rect.half_dim.x*2.0, rect.half_dim.y*2.0, color);
}

