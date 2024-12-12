use std::default;

use crate::options::*;
use macroquad::prelude::*;
use miniquad::*;


pub fn make_mult_blend() -> BlendState {
	BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Value(BlendValue::SourceColor))
}




pub const DEF_VERTEX: &str = r"#version 100
precision lowp float;
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
	gl_Position = Projection * Model * vec4(position, 1);

	color = color0 / 255.0;

	uv = texcoord;
}
";

pub const BRIGHTNESS_VERTEX: &str = r"#version 100
precision lowp float;
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

uniform float brightness;

void main() {
	gl_Position = Projection * Model * vec4(position, 1);

	color = color0 / 255.0;
	color *= brightness;

	uv = texcoord;
}
";

pub const DEF_FRAGMENT: &str = r"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
	vec4 res = texture2D(Texture, uv) * color;
	gl_FragColor = res;
}
";

pub const BLUR_SHADER: &str =r"#version 120
precision lowp float;
varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;
uniform lowp vec2 inv_dsize;

uniform lowp vec2 dithdsize;


const lowp float ditherPattern[4] = float[4](
	1.00, 0.50,
	0.25, 0.75
);

void main() {
	vec4 res = texture2D(Texture, uv) * 0.3;
	for (float dir = -1.0; dir < 1.1; dir += 2.0) {
		res += texture2D(Texture, uv + inv_dsize*vec2(dir, 0.0))*0.1;
		res += texture2D(Texture, uv + inv_dsize*vec2(0.0, dir))*0.1;
		res += texture2D(Texture, uv + inv_dsize*vec2(dir, dir))*0.075;
		res += texture2D(Texture, uv + inv_dsize*vec2(dir, -dir))*0.075;
	}
	lowp vec2 pp = vec2(uv * dithdsize);
	pp = floor(pp);

	gl_FragColor = res * color + 0.90*ditherPattern[int(mod(pp.x, 2.0) + mod(pp.y, 2.0)*2.0)] / (255.0);
}
";

pub const POST_PROCESSING: &str = r"#version 120
precision lowp float;
varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;

float square(float x) {return x*x;}

const lowp float VIN_AM = 0.45;
const lowp float BRIGHTNESS = 1.1;
uniform lowp vec2 screenSize;
const lowp float ditherPattern[4] = float[4](
	1.00, 0.50,
	0.25, 0.75
);

const lowp float sharpAm = 0.2;

void main() {
	float vin = 1.0 - VIN_AM*square(uv.x*2.0-1.0);
	vin *= 1.0 - VIN_AM*square(uv.y*2.0-1.0);

	vec4 res = texture2D(Texture, uv) * color;

	vec2 inv_size = 2/screenSize;
	vec3 sharpColor = vec3(0.0, 0.0, 0.0);
	for (float i = -1; i <= 1.5; i+=2.0) {
		sharpColor += texture2D(Texture, uv + inv_size * vec2(0.0, i)).rgb * 0.1875;
		sharpColor += texture2D(Texture, uv + inv_size * vec2(i, 0.0)).rgb * 0.1875;
		sharpColor += texture2D(Texture, uv + inv_size * vec2(i, i)).rgb * 0.0625;
		sharpColor += texture2D(Texture, uv + inv_size * vec2(-i, i)).rgb * 0.0625;
	}
	res.rgb -= (sharpColor - res.rgb) * sharpAm;
	res.rgb = res.rgb + res.rgb * res.rgb;
	res.rgb *= 0.5 * vin;

	lowp vec2 pp = vec2(uv * screenSize);
	pp = floor(pp);

	
	
	gl_FragColor = res * BRIGHTNESS + 0.90*ditherPattern[int(mod(pp.x, 2.0) + mod(pp.y, 2.0)*2.0)] / 255.0;

}
";

const MAX_LIGHT_BRIGHTNESS:f32 = 1.0;

pub fn create_light_mat () -> Material {
	let mat = load_material(ShaderSource::Glsl { vertex: BRIGHTNESS_VERTEX, fragment: DEF_FRAGMENT }, MaterialParams{
		pipeline_params: PipelineParams{
			cull_face: CullFace::Nothing,
			depth_test: Comparison::Always,
			depth_write: false,
			color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)),
			..Default::default()
		},
		uniforms: vec!(("brightness".to_string(), UniformType::Float1)),
		..Default::default()
	}).unwrap();
	mat.set_uniform("brightness", 0.55f32);
	mat
}

pub fn create_shadow_mat () -> Material {
	let mat = load_material(ShaderSource::Glsl { vertex: DEF_VERTEX, fragment: BLUR_SHADER }, MaterialParams{
		pipeline_params: PipelineParams{
			cull_face: CullFace::Nothing,
			depth_test: Comparison::Always,
			depth_write: false,
			color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::DestinationColor), BlendFactor::Value(BlendValue::SourceColor))),
			..Default::default()
		},
		uniforms: vec![("inv_dsize".to_string(), UniformType::Float2), ("dithdsize".to_string(), UniformType::Float2)],

		..Default::default()
	}).unwrap();
	mat.set_uniform("inv_dsize", 1.5*vec2(1.0/(SHADOWMAP_DIM.x as f32), 1.0/(SHADOWMAP_DIM.y as f32)));
	mat.set_uniform("dithdsize", SCREEN_DIM.as_vec2());
	mat
}

pub fn create_screen_mat() -> Material {
	let mat = load_material(
	ShaderSource::Glsl { vertex: DEF_VERTEX, fragment: POST_PROCESSING },
	MaterialParams{
		pipeline_params: PipelineParams{
			cull_face: CullFace::Nothing,
			depth_test: Comparison::Always,
			depth_write: false,
			..Default::default()
		},
		uniforms: vec![("screenSize".to_string(), UniformType::Float2)],
		..Default::default()
	}).unwrap();
	mat.set_uniform("screenSize", SCREEN_DIM.as_vec2());
	mat
}