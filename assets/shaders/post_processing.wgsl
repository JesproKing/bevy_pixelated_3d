#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::{
    view::View,
    globals::Globals,
}
struct PostProcessSettings {
    show_depth: u32,
    show_normals: u32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var palette_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var<uniform> settings: PostProcessSettings;
@group(0) @binding(4) var depth_texture: texture_depth_2d;
@group(0) @binding(5) var prepass_normal_texture: texture_2d<f32>;
@group(0) @binding(6) var<uniform> view: View;

const lookupSize = 64.0;
const errorCarry = 0.3;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<i32>(in.position.xy);

    let uv_offsets = array<vec2<i32>, 4>(
	    vec2(uv + vec2(0, -1)),
	    vec2(uv + vec2(0, 1) ),
	    vec2(uv + vec2(1, 0) ),
	    vec2(uv + vec2(-1, 0)) 
    );

    var outline_mask = textureLoad(prepass_normal_texture, uv, 0).a;
	outline_mask = floor(outline_mask);
	
	// Roberts Cross edge detection
	// Edge detection with Depth
    // var depth = textureLoad(depth_texture, uv, 0);
	let depth = -get_linear_depth(uv, view.clip_from_view, outline_mask) * 0.5;
	let d = get_depth_difference(uv, outline_mask);
    let depth_diff = d.x;
    let neg_depth_diff = d.y;

	// Edge detection with Normals
	var normal_diff = 0.;
	let normal_edge_bias = vec3(1.0, 1.0, 1.0);
	let normal = get_normal(uv, outline_mask);
	
	for (var i = 0; i < 4; i++){
		let n_off = get_normal(uv_offsets[i], outline_mask);
		normal_diff += normal_edge_indicator(normal_edge_bias, normal, n_off, depth_diff);
	}
	normal_diff = smoothstep(0.5, 0.9, normal_diff);
	normal_diff = clamp(normal_diff - neg_depth_diff, 0.0, 1.0);


    var texel = textureSample(screen_texture, texture_sampler, in.uv);
	// ALBEDO = texture(SCREEN_TEXTURE, SCREEN_UV).rgb; 
	let line_mask = clamp(0.1, 0.7, (depth_diff + normal_diff * 5.0));
    
    if settings.show_depth == 1u {
        return vec4(depth, depth, depth, 1.);
    } else if settings.show_normals == 1u {
        return vec4(depth_diff, depth_diff, depth_diff, 1.);
    }

    let line_highlight = 1.2;
    let line_shadow = 0.55;
    // let lum = 0.2126 * texel.r + 0.7152 * texel.g + 0.0722 * texel.b;
    // texel = vec4(lum, lum, lum, 1.);
    texel = quantize(texel, 8);
    texel += texel * clamp((normal_diff - depth_diff), 0.0, 1.0) * line_highlight;
    texel -= texel * depth_diff * line_shadow;
    return texel;
}

fn quantize(texel: vec4<f32>, amount: i32) -> vec4<f32> {
	let scale = exp2(f32(amount)) - 1.0;
	return floor(texel * scale + 0.5f) / scale;
}

fn get_depth_difference(uv: vec2<i32>, outline_mask: f32) -> vec2<f32>{
    let uv_offsets = array<vec2<i32>, 4>(
	    vec2(uv + vec2(0, -1)),
	    vec2(uv + vec2(0, 1) ),
	    vec2(uv + vec2(1, 0) ),
	    vec2(uv + vec2(-1, 0)) 
    );

    var depth_diff = 0.0;
	var neg_depth_diff = 0.7;
	let depth = get_linear_depth(uv, view.clip_from_view, outline_mask);
	
	for (var i = 0; i < 4; i++){
		let d_off = get_linear_depth(uv_offsets[i], view.clip_from_view, outline_mask);
		depth_diff += clamp(d_off - depth, 0.0, 1.0);
		neg_depth_diff += depth - d_off;
	}

    neg_depth_diff = clamp(neg_depth_diff, 0.0, 1.0);
	neg_depth_diff = clamp(smoothstep(0.9, 1.0, neg_depth_diff) * 10.0 , 0.0, 1.0);
	depth_diff = smoothstep(0.2, 0.3, depth_diff);

    return vec2(depth_diff, neg_depth_diff);
}

fn normal_edge_indicator(
    normal_edge_bias: vec3<f32>, normal: vec3<f32>, neighbour_normal: vec3<f32>, depth_diff: f32
) -> f32 {
    let normal_diff = dot(normal - neighbour_normal, normal_edge_bias);
    let normal_indicator = clamp(smoothstep(-.01, .01, normal_diff), 0., 1.);
    let depth_indicator = clamp(sign(depth_diff * 0.25 + 0.0025), 0., 1.);
    return (1. - dot(normal, neighbour_normal)) * depth_indicator * normal_indicator;
}

fn get_normal(uv: vec2<i32>, mask: f32) -> vec3<f32> {
    let prepass_normal_sample = textureLoad(prepass_normal_texture, uv, 0);
    let prepass_normal = prepass_normal_sample.xyz  * 2.0 - vec3(1.0) * mask;
    return prepass_normal;
}

fn get_depth(uv: vec2<i32>) -> f32 {
    var depth = textureLoad(depth_texture, uv, 0);
    depth = (depth - 0.45) * 4;
    if depth < 0. {
        depth = 0.;
    }
    if depth > 1. {
        depth = 1.;
    }
    return depth;
}

fn get_linear_depth(s_uv: vec2<i32>, inv_projection_mat: mat4x4<f32>, mask: f32) -> f32{
    let depth = textureLoad(depth_texture, s_uv, 0) * mask;
    // let ndc = vec3<f32>(
    //     f32(s_uv.x) * 2.0f - 1.0f,
    //     f32(s_uv.y) * 2.0f - 1.0f,
    //     depth
    // );
    // var view = inv_projection_mat * vec4(ndc, 1.0);
    return -(inv_projection_mat[3].z - depth) / inv_projection_mat[2].z;
}

fn smoothSign(x: f32, radius: f32) -> f32 {
    return smoothstep(-radius, radius, x) * 2.0 - 1.0;
}
