
use core::f32;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default;

use macroquad::prelude::*;
use crate::collision::*;
use crate::enemy::*;
use crate::go_helpers::*;

use crate::parser::*;

use crate::partical_system::Partical;
use crate::player;
use crate::player::*;
use crate::staticobj::*;
use crate::wall_man::*;
use crate::partical_system::{ParticalSystem, ParticalRenderer};

use crate::options::*;
use crate::custom_text::*;

use crate::light_sources;

const DEF_TILE:Vec2 = vec2(0.0, 3.0);

const TILE_SIZE: f32 = 16.0;
const TILE_VEC: Vec2 = Vec2::splat(TILE_SIZE);

const CAM_SPEED: f32 = 8.0;

const TEXT_FADE_SPEED: f32 = 3.0;

#[derive(Copy, Clone, PartialEq)]
enum GameState {
	Paused,
	Unpaused,
	ForcePaused,
}

pub struct GameWorld {
	assets: AssetManager,

	anim_timer: f32,

	// gameobjects
	lights: Vec<(Vec2, char)>,

	triggers: Vec<Trigger>,

	breakable_walls: Vec<CollBox>,
	ghost_blocks: GhostBlocks,
	
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
	time_until_level_switch: Option<(f32, bool)>,


	hud: HUD,
	//wall_stuff: Vec<MultiCollider<CollBox>>
	game_state: GameState,
	wall_man: WallMan<Vec2>,

	part_sys: ParticalSystem,
}

impl GameWorld {
	pub fn new(assets: AssetManager) -> Self {
		let gw = Self {
			// static objects
			lights: Vec::new(),

			triggers: Vec::new(),
			breakable_walls: Vec::new(),
			ghost_blocks: GhostBlocks::new(Vec2::splat(8.0)),

			// things that kill you
			enemies: Vec::new(),

			bullets: Vec::new(),
			
			// single instance objects
			player: Player::new(),
			level_exit: Option::None,

			note: Option::None,

			cam_position: Vec2::ZERO,

			// book keeping and things that make the levels work
			world_size: CAM_DIM,

			time_speed: 1.0,
			tileset: 0,
			level_index: 0,
			load_new_level: false,
			level_blueprint: Vec::new(),


			time_until_level_switch: None,


			hud: HUD::new(&assets, ""),
			game_state: GameState::Unpaused,
			assets: assets,

			anim_timer: 0.0,

			wall_man: WallMan::new(generate_multi_col_grid(Vec2::ZERO, Vec2::splat(128.0), uvec2(5,3)), DEF_TILE),

			part_sys: ParticalSystem::new(),
		};
		gw
	}

	pub fn setup(&mut self) {
		// resets the game world
		self.player = Player::new();

		self.lights.clear();

		self.triggers.clear();
		self.breakable_walls.clear();

		self.enemies.clear();
		self.bullets.clear();
		self.ghost_blocks.clear();
		self.level_exit = None;
		self.note = None;

		let wm_grid_size = 64.0;
		self.wall_man = WallMan::new(generate_multi_col_grid(Vec2::splat(-8.0),
			Vec2::splat(wm_grid_size),
			UVec2::ONE+(self.world_size/wm_grid_size).as_uvec2()
		),DEF_TILE);
		
		//self.world_size = CAM_DIM;

		// parses the layout
		for bp_tile in self.level_blueprint.as_slice() {
			let ent_pos = bp_tile.pos;
			//self.world_size.x = self.world_size.x.max(ent_pos.x);
			//self.world_size.y = self.world_size.y.max(ent_pos.y);
			match bp_tile.ty {
				'p' => 	self.player.cbox.pos = ent_pos, // player

				'#' => self.wall_man.add_wall(collbox!(ent_pos, TILE_VEC*0.5)),// wall

				'E' => self.enemies.push(new_angry_ghost(ent_pos)), // enemies

				'T' => self.enemies.push(new_turret(ent_pos)),

				'S' => self.enemies.push(new_stalker(ent_pos)),

				'l' => self.lights.push((ent_pos, 'c')),

				'g' => self.ghost_blocks.add(ent_pos),

				'^' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(7.0)), TriggerType::Trap(0.0))),

				'f' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(46.0*0.5)), TriggerType::FlyBox)), // fly_boxes
				
				'*' => self.triggers.push(Trigger::new(collbox!(ent_pos, Vec2::splat(6.0)), TriggerType::Spikes)), // death_box

				'/' => self.breakable_walls.push(collbox!(ent_pos, TILE_VEC*0.5)),

				'X' => self.level_exit = Some((collbox!(ent_pos + vec2(0.0, 1.0) * (TILE_SIZE / 2.0 - 4.0 ) as f32, (4.0, 4.0)), 0.0)), // exit

				'N' => self.note = Some(ent_pos), // Notes

				_ => {}
			}
		}

		// sets the camera to the right place
		self.cam_position = self.player.cbox.pos;
		self.constrain_cam();

		// ensures spikes are never blocked by other triggers
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

		self.wall_man.cull(); // removes empty multi colliders

		// assigns a sprite to each tile
		self.wall_man.assign_tile_types(|exp, coll| {
			//return vec2(0.0, 3.0);
			match (coll.x, coll.y) {
				(true, true) => (ivec2(1, 1) - exp).as_vec2(),
				(true, false) => if exp.x == 0 {vec2(2.0, 3.0)} else {DEF_TILE},
				(false, true) => if exp.y == 0 {vec2(1.0, 3.0)} else {DEF_TILE},
				(_) => vec2(0.0, 3.0),
			}
		}, Vec2::splat(8.5), Vec2::splat(0.1), Some(self.world_size));
	
	for i in 0..self.lights.len() {
		self.lights[i].1 = light_sources::assign_light_type(self.lights[i].0, &self.wall_man.wall_colliders);
	}
	}

	
	pub fn update(&mut self, dt:f32) {
		if is_key_pressed(KeyCode::Escape) {
			self.game_state = match self.game_state {
				GameState::Unpaused => GameState::Paused,
				GameState::Paused => GameState::Unpaused,
				_ => self.game_state,
			}
		}

		macro_rules! kill_player {
			() => {
				self.player.kill(&mut self.part_sys);
				self.queue_level_load(false);
			}
		}

		self.anim_timer += dt;
		self.anim_timer %= 4.0;

		if  self.game_state != GameState::Unpaused {
			return;
		}

		let mut queue_reset = false;
		// player logic
		self.player.update(dt * self.time_speed);
		let mut player_in_trap = false;
		self.player.physics_update_mc(&self.wall_man.wall_colliders.as_slice(), &self.breakable_walls, &self.ghost_blocks, dt * self.time_speed);
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
			kill_player!();
		}

		// keeps the player inside the game world
		self.player.cbox.pos.y = 0.0f32.max(self.player.cbox.pos.y);
		self.player.cbox.pos.x = self.player.cbox.pos.x.clamp(0.0, self.world_size.x);

		// camera logic
		self.cam_position = move_toward_ex_2D(self.cam_position, self.player.cbox.pos, dt*CAM_SPEED);
		self.constrain_cam();
		
		

		// enemy logic
		let mut enemy_in_nm = false;
		run_and_remove(&mut self.enemies, |enemy| {
			let mut dead = false;
			let mut in_fb = false;
			// removes enemies that collide with triggers
			Trigger::coll_check(&mut self.triggers, &enemy.cbox, |trigger|{
				match trigger.trigger_type {
					TriggerType::Spikes => {
						dead = true;
						create_enemy_death_particals(&mut self.part_sys, enemy.cbox.pos);
					},
					
					TriggerType::FlyBox => in_fb = true,
					TriggerType::Trap(_) => enemy_in_nm = true,
					_ => {}
				}
				true
			});
			enemy.update(&self.player.cbox, &mut self.bullets, in_fb,player_in_trap, dt * self.time_speed);
			
			return dead;
		});

		// ghostblock stuff
		if self.player.check_ghost_block_kill(&self.ghost_blocks) {
			kill_player!();
			//println!("partical count: {}", self.part_sys.get_count());
		}
		self.ghost_blocks.update(player_in_trap || enemy_in_nm, dt);

		// bullet logic
		run_and_remove(&mut self.bullets, |bullet|{
			bullet.update(dt);
			// check for wall collisions
			if query(&self.wall_man.wall_colliders.as_slice(), |mc| {
				mc.check_intersection(&bullet.coll.to_box())
			} || if self.ghost_blocks.get_block_effect() == GhostBlockEffect::Collide {
				query(self.ghost_blocks.get_block_slice(), |gb| {check_circle_box_intersection(&bullet.coll, &gb)})
			} else {false}) {
				self.part_sys.create_partical(4, bullet.coll.pos, 1.0, bullet.coll.pos-bullet.motion * 0.1, 5.0, 1.0, 1, ParticalRenderer::Circle);
				return true;
			}
			bullet.life_time_update(dt)
		});

		// enemy bullet interactions
		for e in (0..self.enemies.len()).rev() {
			// skips enemies that are immune to bullets
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
		
		if query(self.bullets.as_slice(), |bullet|
			{check_circle_box_intersection(&bullet.coll, &self.player.cbox)}
		) {
			kill_player!();
		}
		
		// handles breakable walls
		run_and_remove(&mut self.breakable_walls, |br| {
			let destroyed = 
			query(&self.bullets, |bu| {check_circle_box_intersection(&bu.coll, &br)});

			if destroyed {
				self.part_sys.create_partical(15, br.pos, TILE_SIZE/2.0, br.pos - vec2(0.0, 16.0), TILE_SIZE, 1.0, 2, ParticalRenderer::Dust)
			}

			destroyed
		});


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

		self.part_sys.update(dt);

		self.hud.update(
			match self.note {
				Some(note) => {
					self.player.cbox.check_intersection(&collbox!(note, (24.0, 32.0)))
				}
				None => {false}
			}, dt);

		if let Some((time_til_switch, next_level)) = self.time_until_level_switch {
			self.time_until_level_switch = Some((time_til_switch - dt, next_level));
		}

		if queue_reset {
			kill_player!();
		}
	}

	

	pub fn draw(&self) {
		draw_background(&self.assets.world_images, Some(Rect::new(16.0 + 48.0 * self.tileset as f32, 16.0, 16.0, 16.0)), self.cam_position);

		light_sources::draw_light_sources(&self.lights, &self.assets.world_images);

		match &self.level_exit { // draws door
			Some(exit) => {
				let offset = TILE_SIZE / 2.0 - exit.0.half_dim.y;
				draw_centered_texture(&self.assets.world_images, exit.0.pos - vec2(0.0, offset), EXIT_COLOR, DrawTextureParams{
					dest_size: Some(TILE_VEC),
					source: if !exit.0.check_intersection(&self.player.cbox){
						Some(Rect::new(0.0, 112.0, 16.0, 16.0))
					} else {
						Some(Rect::new(16.0, 112.0, 16.0, 16.0))
					},
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

		self.ghost_blocks.draw(&self.assets.world_images);

		for i in 0..self.enemies.len() {
			let enemy = &self.enemies[i];
			enemy.draw(&self.assets.dangers, self.player.cbox.pos);
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

		self.part_sys.draw(&self.assets.particals);
	}

	pub fn draw_lights (&self) {
		let light_tex = &self.assets.lights_tex;
		draw_centered_texture(&light_tex, self.player.cbox.pos, PLAYER_GLOW, DrawTextureParams{
			dest_size: Some(Vec2::splat(64.0+32.0)),
			source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),

			..Default::default()
		});

		if let Some(note) = self.note {
			draw_centered_texture(&light_tex, note, GRAY, DrawTextureParams{
				dest_size: Some(Vec2::splat(64.0+16.0)),
				source: Some(Rect::new(66.0, 0.0, 32.0, 32.0)),
				rotation: self.anim_timer * f32::consts::PI / 4.0,
				pivot: Some(note),
				..Default::default()
			});
		}

		if let Some((coll, _)) = self.level_exit {
			draw_centered_texture(&light_tex, coll.pos, GRAY, DrawTextureParams{
				dest_size: Some(Vec2::splat(128.0)),
				source: Some(Rect::new(66.0, 0.0, 32.0, 32.0)),
				rotation: self.anim_timer * f32::consts::PI / 8.0,
				pivot: Some(coll.pos),
				..Default::default()
			});
		}

		light_sources::draw_lights(&self.lights, &self.assets.lights_tex);

		for i in 0..self.triggers.len() {
			let trigger = &self.triggers[i];

			match trigger.trigger_type {
				TriggerType::Spikes => {
					
					draw_centered_texture(&light_tex, trigger.coll.pos, DANGER_LIGHT, DrawTextureParams{
						dest_size: Some(Vec2::splat(32.0+20.0)),
						source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
						..Default::default()
					})
				}

				TriggerType::FlyBox => {
					draw_centered_texture(&light_tex, trigger.coll.pos, LIGHTGRAY, DrawTextureParams{
						dest_size: Some(Vec2::splat(64.0+32.0)),
						source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
						..Default::default()
					})
				}

				TriggerType::Trap(time) => {
					if time <= 0.01 {continue;}
					draw_centered_texture(&light_tex, trigger.coll.pos, WHITE, DrawTextureParams{
						dest_size: Some(Vec2::splat((time/TRAP_ANIM_TIME)*64.0)),
						source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
						..Default::default()
					})
				}
				_ => {}
			}
		}

		for i in 0..self.enemies.len() {
			let enemy = &self.enemies[i];
			draw_centered_texture(&light_tex, enemy.cbox.pos, DANGER_LIGHT, DrawTextureParams{
				source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
				dest_size: Some(Vec2::splat(ENEMY_DET_SIZE*2.0)),
				..Default::default()
			});
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
				self.load_level(format!("assets/Levels/{}.par", level_file_names[self.level_index as usize]).as_str()).await;
				self.setup();
				return;
			}
		}
		match &self.level_exit {
			Some(exit) => {
				if self.player.cbox.check_intersection(&exit.0) {
					self.queue_level_load(true);
					self.player.disable_player();
				}
			}

			None => {}
		}

		if let Some((time_til_switch, next_level)) = self.time_until_level_switch {
			if time_til_switch < 0.0 {
				if next_level{
					self.time_until_level_switch = None;
					load_next_level!();
				}else {
					self.setup();
					self.time_until_level_switch = None;
					return;
				}
			}
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

				self.hud = HUD::new(&self.assets, &parser.get_as_string_literal_or_def("noteText", "").to_uppercase().replace("\n\n", "\n"));

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
		self.cam_position = self.cam_position.clamp(CAM_DIM, self.world_size - CAM_DIM);
	}

	pub fn set_assets(&mut self, assets:AssetManager) {
		self.assets = assets;
	}

	pub fn queue_level_load(&mut self, load_next_level: bool){
		match self.time_until_level_switch {
			None => self.time_until_level_switch = Some((0.25, load_next_level)),

			_ => {}
		}
	}
}

pub struct AssetManager {
	player_image: Texture2D,
	dangers: Texture2D,
	world_images: Texture2D,
	particals: Texture2D,
	lights_tex: Texture2D,
	font: RFont
}

impl AssetManager {
	pub async fn new() -> Self {
		let player_image: Texture2D = load_texture("assets/Images/Player.png").await.unwrap();
		player_image.set_filter(FilterMode::Nearest);

		let world_images = load_texture("assets/Images/LevelImages.png").await.unwrap();
		world_images.set_filter(FilterMode::Nearest);

		let dangers = load_texture("assets/Images/Dangers.png").await.unwrap();
		dangers.set_filter(FilterMode::Nearest);

		let particals = load_texture("assets/Images/Particals.png").await.unwrap();
		particals.set_filter(FilterMode::Nearest);

		let lights = load_texture("assets/Images/Lights.png").await.unwrap();
		lights.set_filter(FilterMode::Linear);
		
		let font_image = load_texture("assets/Images/Font.png").await.unwrap();
		font_image.set_filter(FilterMode::Nearest);
		let mut hash = HashMap::new();

		format_alphabet(&mut hash, Vec2::ZERO, Vec2::splat(8.0));

		format_characters(&mut hash, "0123456789.,:!'", vec2(0.0, 8.0), Vec2::splat(8.0));
		
		Self {
			player_image: player_image,
			world_images: world_images,
			dangers: dangers,

			particals: particals,

			lights_tex: lights,

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
		self.note_opacity = (self.note_opacity + dt * TEXT_FADE_SPEED *
			if player_reading {1.0} else {-1.0}
		).clamp(0.0, 1.0);
	}

	pub fn draw (&self, assets: &AssetManager, camera_pos: Vec2) {
		let offset = vec2(-64.0, -32.0);
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

	let starting = use_pos - CAM_DIM - Vec2::splat(TILE_SIZE);
	let ending = use_pos + CAM_DIM + Vec2::splat(TILE_SIZE);

	let mut x = starting.x;

	while x < ending.x {
		let mut y = starting.y;
		while y < ending.y {
			draw_centered_texture(&texture, vec2(x, y), BACKGROUND_COLOR, DrawTextureParams{
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

