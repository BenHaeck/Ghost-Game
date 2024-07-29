

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
mod costom_text;

use macroquad::prelude::*;
use game_world::{AssetManager, GameWorld};

fn window_setup() -> Conf {
	Conf {
		window_title: String::from("Uhh"),
		high_dpi: true,
		
		..Default::default()
		
	}
}

#[macroquad::main(window_setup)]
async fn main() {
	set_pc_assets_folder("assets");

	println!("Camera_Dim: {}", options::CAM_DIM);

	let campaign = parser::StringParser::new(load_string("Levels/Campaign.par").await.unwrap().as_str());

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

	let screen = render_target(options::SCREEN_WIDTH, options::SCREEN_HEIGHT);
	screen.texture.set_filter(FilterMode::Nearest);
	let mut camera = Camera2D{
		//viewport: Some((0, 0, 200, 150)),
		zoom: vec2(1.0/options::CAM_WIDTH as f32, 1.0/options::CAM_HEIGHT as f32),
		target: options::CAM_DIM.as_vec2(),
		render_target: Some(screen.clone()),
		..Camera2D::default()
	};
	//let mut gw = game_world::GameWorld::new();
	loop {
		if is_key_pressed(KeyCode::O){
			world.set_assets(AssetManager::new().await);
			world.load_level(&create_level_path(&level_file_names[world.level_index as usize])).await;
		}

		if get_frame_time() > 0.0001 {
			world.update(get_frame_time().min(0.15));
		}
		world.handle_levels(&level_file_names).await;
		camera.target = (world.cam_position * 2.0).round() / 2.0;
		
		//camera.zoom = vec2(1.0/screen.texture.width(), 1.0/screen.texture.height());
		
		clear_background(BLACK);

		// draw to the render_texture
		set_camera(&camera);
		clear_background(DARKGRAY);
		world.draw();

		world.draw_hud();
		
		//draw_text(format!("FPS: {}", get_fps()).as_str(), world.player.cbox.pos.x, world.player.cbox.pos.y, 16.0, RED);
		// draw_texture to screen
		set_default_camera();
		let(screen_pos, screen_scale) = scale_to_fit(screen.texture.size(), vec2(screen_width(), screen_height()));
		draw_texture_ex(&screen.texture, screen_pos.x, screen_pos.y, WHITE, DrawTextureParams{
			dest_size: Some(screen.texture.size() * screen_scale),
			..Default::default()
		});
		
		
		
		

		next_frame().await;
	}
}

fn create_level_path (level_name: &str) -> String {
	format!("Levels/{}.par", level_name)
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