

pub mod parser;

pub mod enemy;
pub mod animation;
pub mod collision;
pub mod go_helpers;
pub mod game_world;
pub mod player;
pub mod options;
mod wall_man;
pub mod staticobj;
pub mod StaticObj;
pub mod partical_system;
mod custom_text;

mod custom_shader;

mod light_sources;

use light_sources::*;

use custom_shader::*;


use std::{clone, result};

use custom_shader::{BLUR_SHADER, DEF_FRAGMENT};
use go_helpers::draw_centered_texture;
use macroquad::{material, prelude::*};
use game_world::{AssetManager, GameWorld};
use options::{AMBIENT_LIGHT, CAM_DIM, CAM_HEIGHT, CAM_WIDTH, LIGHT_DIV, SCREEN_DIM, SHADOW_CAM_DIM};
use partical_system::Partical;

fn window_setup() -> Conf {
	Conf {
		window_title: String::from("The Ghostly Game"),
		high_dpi: true,
		
		..Default::default()
		
	}
}

#[macroquad::main(window_setup)]
async fn main() {
	match run().await {
		Ok(()) => {return;},

		Err(e) => {
			loop{
				clear_background(DARKGRAY);
				draw_text(&e, 6.0, 16.0, 32.0, WHITE);
				if (is_key_down(KeyCode::Escape)) {break;}
				next_frame().await;
			}
		}
	}
}

async fn run() -> Result<(), String>{
	let mut fullscreen: bool = false;
	//set_pc_assets_folder("assets");

	macro_rules! safe_unwrap {
		($res:expr) => {
			{
				let r = $res;
				match r {
					Ok(a) => {a},

					Err(_) => {return Result::<(), String>::Err("File_error".to_string());}
				}
			}
		};
	}

	macro_rules! s_load_text {
		($fname:expr) => {
			match load_string($fname).await {
				Ok(s) => {s},

				Err(_) => {return Result::Err(($fname.to_string()))}
			}
		}
	}

	// load assets & set up world
	let campaign = parser::StringParser::new(s_load_text!("assets/Levels/Campaign.par").as_str());

	let level_file_names: Vec<String> = campaign.get_as_strings("Levels");
	
	if campaign.get_int_or_def("printLevelNames", 0) == 1 {
		for i in 0..level_file_names.len() {
			print!("{}, ", level_file_names[i]);
		}
		println!();
	}
	
	let assets = AssetManager::new().await;
	let mut world = GameWorld::new(assets);
	{
		let start_ind = campaign.get_int_or_def("startAt", 0).abs();
		world.level_index = start_ind;
		world.load_level(create_level_path(level_file_names[start_ind as usize].as_str()).as_str()).await;
	}
	world.setup();

	// setup render texture and camera
	let screen = render_target(options::SCREEN_WIDTH, options::SCREEN_HEIGHT);
	screen.texture.set_filter(FilterMode::Nearest);
	let mut camera = Camera2D{
		//viewport: Some((0, 0, 200, 150)),
		zoom: vec2(1.0/options::CAM_WIDTH as f32, 1.0/options::CAM_HEIGHT as f32),
		target: options::CAM_DIM,
		render_target: Some(screen.clone()),
		..Camera2D::default()
	};

	let shadow_map = render_target(options::SHADOWMAP_DIM.x, options::SHADOWMAP_DIM.y);
	shadow_map.texture.set_filter(FilterMode::Linear);
	let mut shadow_cam = Camera2D{
		zoom: camera.zoom / 2.0,
		
		render_target: Some(shadow_map.clone()),
		..Default::default()
	};

	let light_mat = create_light_mat();
	let shadow_mat = create_shadow_mat();
	let screen_mat = create_screen_mat();
	// loop
	loop {
		if is_key_pressed(KeyCode::O){
			world.set_assets(AssetManager::new().await);
			world.load_level(&create_level_path(&level_file_names[world.level_index as usize])).await;
		}

		if is_key_pressed(KeyCode::F11) {
			fullscreen = !fullscreen;
			set_fullscreen(fullscreen);
		}

		/*if is_key_down(KeyCode::Escape) {
			break;
		}*/

		if get_frame_time() > 0.0001 {
			world.update(get_frame_time().min(0.15));
		}
		world.handle_levels(&level_file_names).await;
		camera.target = (world.cam_position * 2.0).round() / 2.0;
		shadow_cam.target = (world.cam_position * 2.0).round() / 2.0;
		
		//camera.zoom = vec2(1.0/screen.texture.width(), 1.0/screen.texture.height());
		
		clear_background(BLACK);

		// draws lights
		set_camera(&shadow_cam);
		clear_background(AMBIENT_LIGHT);
		gl_use_material(&light_mat);
		world.draw_lights();
		gl_use_default_material();
		set_camera(&camera);
		// draw to the render_texture
		world.draw();

		gl_use_material(&shadow_mat);
		draw_centered_texture(&shadow_map.texture, world.cam_position, WHITE, DrawTextureParams{
			dest_size: Some(SCREEN_DIM.as_vec2()),
			..Default::default()
		});
		gl_use_default_material();
		
		world.draw_hud();
		
		//draw_text(format!("FPS: {}", get_fps()).as_str(), world.player.cbox.pos.x, world.player.cbox.pos.y, 16.0, RED);
		// draw_texture to screen
		set_default_camera();
		let(screen_pos, screen_scale) = scale_to_fit(screen.texture.size(), vec2(screen_width(), screen_height()));
		screen_mat.set_uniform("screenSize", vec2(screen_width(), screen_height()));
		gl_use_material(&screen_mat);
		draw_texture_ex(&screen.texture, screen_pos.x, screen_pos.y, WHITE, DrawTextureParams{
			dest_size: Some(screen.texture.size() * screen_scale),
			..Default::default()
		});
		gl_use_default_material();
		
		
		

		next_frame().await;
	}

	return Result::Ok(());
}

fn create_level_path (level_name: &str) -> String {
	format!("assets/Levels/{}.par", level_name)
}

fn scale_to_fit(src_size: Vec2, dest_size:Vec2) ->(Vec2, f32) {
	let mut scale = dest_size.x / src_size.x;

	if scale * src_size.y <= dest_size.y {
		(vec2(0.0, dest_size.y - src_size.y * scale)*0.5, scale)
	}
	else {
		scale = dest_size.y / src_size.y;
		(vec2(dest_size.x - src_size.x * scale, 0.0)*0.5, scale)
	}
}

/*fn calc_viewport(src_size:Vec2, dest_size:Vec2) ->(i32, i32, i32, i32) {
	let(pos, scale) = scale_to_fit(src_size, dest_size);
	let f_dest_size = dest_size * scale;

	(pos.x as i32, pos.y as i32, f_dest_size.x as i32, f_dest_size.y as i32)
}*/