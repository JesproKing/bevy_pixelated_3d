#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_pbr::{
    prepass_utils,
}

struct PostProcessSettings {
    show_depth: u32,
    show_normals: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var palette_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var<uniform> settings: PostProcessSettings;
@group(0) @binding(4) var depth_texture: texture_depth_2d;
@group(0) @binding(5) var prepass_normal_texture: texture_2d<f32>;

const lookupSize = 64.0;
const errorCarry = 0.3;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var texel = textureSample(screen_texture, texture_sampler, in.uv);
    let uv = vec2<i32>(in.position.xy);
    
    // let normal_edge_coefficient = (smoothSign(t_lum - .3, .1) + .7) * .25;
    // let depth_edge_coefficient = (smoothSign(t_lum - .3, .1) + .7) * .4;
    let normal_edge_coefficient = .2;
    let depth_edge_coefficient = .4;

    let dei = depth_edge_indicator(uv);
    let nei = normal_edge_indicator(uv);

    var coefficient = (1.0 + normal_edge_coefficient * nei);
    if dei > 0.0 {
        coefficient = (1.0 - depth_edge_coefficient * dei);
    } else if dei < 0.0 {
        coefficient = 1.;
    }

    // texel = quantize(texel);
    // texel = dither(texel, uv, in.uv);
    if coefficient >= 1. {
        texel = dither(texel, uv, in.uv);
    //     texel = dither(texel, uv, in.uv);
    }
    texel = quantize(texel);

    return texel * coefficient;
}

fn quantize(color: vec4<f32>) -> vec4<f32>{
    let n = 7.0;
    let r = floor(color.r * (n - 1) + 0.5) / (n-1);
    let g = floor(color.g * (n - 1) + 0.5) / (n-1);
    let b = floor(color.b * (n - 1) + 0.5) / (n-1);
    return vec4(r,g,b,1.0);
}

fn dither(color: vec4<f32>, uv: vec2<i32>, screen_uv: vec2<f32>) -> vec4<f32>{
    var texel = color;
    let size = 1. / 360.;
    let l = textureSample(screen_texture, texture_sampler, screen_uv + vec2(-size, 0.));
    let r = textureSample(screen_texture, texture_sampler, screen_uv + vec2(size, 0.));
    let d = textureSample(screen_texture, texture_sampler, screen_uv + vec2(0., -size));
    let u = textureSample(screen_texture, texture_sampler, screen_uv + vec2(0., size));
    // var xError = vec4(0.0,0.0,0.0,0.0);
    // for(var xLook=0.0; xLook<lookupSize; xLook+= 1.0){
    //     var sample = textureSample(screen_texture, texture_sampler, screen_uv + vec2((-lookupSize+xLook)*size,0));
    //     sample += xError;
    //     var q = quantize(sample);
    //     xError = (sample - q)*errorCarry;
    // }
    
    // var yError = vec4(0.0,0.0,0.0,0.0);
    // for(var yLook=0.0; yLook<lookupSize; yLook+= 1.0){
    //     var sample = textureSample(screen_texture, texture_sampler, screen_uv + vec2(0,(-lookupSize+yLook)*size));
    //     sample += yError;
    //     var q = quantize(sample);
    //     yError = (sample - q)*errorCarry;
    // }
    // 
    // texel += xError*0.5 + yError*0.5;

    var diff = texel * 4.;
    diff -= l;
    diff -= r;
    diff -= u;
    diff -= d;

    // let k = texel - quantize(texel);

    // if diff != 0 {
    //     texel += diff * 0.1;
    // }
    return texel - diff * 0.1;
}

fn ditherv1(color: vec4<f32>, uv: vec2<i32>, screen_uv: vec2<f32>) -> vec4<f32>{
    let s = 0.1;
    var old_pixel = color;//textureSample(screen_texture, texture_sampler, screen_uv);
    var new_pixel = quantize(old_pixel);
    let err = old_pixel - new_pixel;
    var texel = color; //old_pixel + vec4(err.r * 1./16., err.g * 1./16., err.b * 1./16., 1.0);

    let x = uv.x % 4;
    let y = uv.y % 4;
    let pos = x + y*4;
    var m = bayer(pos);
    m *= 1/16.0;
    m -= 0.5;
    texel += vec4(m,m,m,1.0) * s;
    return texel;
}

fn bayer(pos: i32) -> f32{
    var bayer4 = array<f32, 16>(
        0, 8, 2, 10,
        12, 4, 14, 6,
        3, 11, 1, 9,
        15, 7, 13, 5
    );
    return bayer4[pos];
}

fn hsl_to_rgb(hsl: vec4<f32>) -> vec4<f32> {
    let h = hsl.r;
    let s = hsl.g;
    let l = hsl.b;
    var r = 0.;
    var g = 0.;
    var b = 0.;
    let c = ( 1. - abs(2. * l - 1.)) * s;
    let x = c * (1. - abs((h % 2.) - 1.));
    let m = l - c/2.;
    if h >= 0. && h < 1. {
        r = c;
        g = x;
    } else if h >= 1. && h < 2. {
        r = x;
        g = c;
    } else if h >= 2. && h < 3. {
        g = c;
        b = x;
    } else if h >= 3. && h < 4. {
        g = x;
        b = c;
    } else if h >= 4. && h < 5. {
        r = x;
        b = c;
    } else if h >= 5. && h <= 6. {
        r = c;
        b = x;
    }
    r += m;
    g += m;
    b += m;
    return vec4(r, g, b, hsl.a);
}

fn rgb_to_hsl(rgb: vec4<f32>) -> vec4<f32>{
    var h = 0.;
    var s = 0.;
    var l = 0.;
    var c_min = min(rgb.r, min(rgb.g, rgb.b));
    var c_max = max(rgb.r, max(rgb.g, rgb.b));
    l = (c_max + c_min) / 2.;
    if c_max > c_min {
        let c_delta = c_max - c_min;

        if l < .0 {
            s = c_delta / (c_max + c_min);
        } else {
            s = c_delta / (2.0 - c_max + c_min);
        }

        if (rgb.r == c_max) {
            h = (rgb.g - rgb.b) / c_delta;
        } else if ( rgb.g == c_max) {
            h = 2.0 + ( rgb.b - rgb.r) / c_delta;
        } else {
            h = 4.0 + ( rgb.r - rgb.g) / c_delta;
        }

        if h < 0.0 {
            h += 6.;
        }
        if h > 6.0 {
            h = 6.0;
        }
    }
    return vec4(h,s,l,rgb.a);
}

fn get_nearest_color(uv: vec2<i32>, screen_uv: vec2<f32>) -> vec4<f32> {
    let offset = 1;        
    let left_coord      = vec2(uv + vec2(-offset,       0));
	let right_coord     = vec2(uv + vec2( offset,       0));
	let up_coord        = vec2(uv + vec2( 0,       offset));
	let down_coord      = vec2(uv + vec2( 0,      -offset));

    let p = get_depth(uv);
    let l = get_depth(left_coord);
    let r = get_depth(right_coord);
    let u = get_depth(up_coord);
    let d = get_depth(down_coord);

    let m = max(max(max(max(l,r),u),d), p);

    var front_color = textureSample(screen_texture, texture_sampler, screen_uv);
    let size = 1. / 90.;
    if m > 0.0 && m != p {
        if u == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(0., size));
        } else if l == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(-size, 0.));
        } else if r == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(size, 0.));
        } else  if d == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(0., -size));
        }
    }
    return front_color;
}

fn get_normal_color(uv: vec2<i32>, screen_uv: vec2<f32>) -> vec4<f32> {
    let offset = 1;        
    let left_coord      = vec2(uv + vec2(-offset,       0));
	let right_coord     = vec2(uv + vec2( offset,       0));
	let up_coord        = vec2(uv + vec2( 0,       offset));
	let down_coord      = vec2(uv + vec2( 0,      -offset));

    var depth = get_depth(uv);
    var normal = get_normal(uv);
    let bias = vec3(-1.,1.,1.);
    let p = neighbor_normal_edge_indicator(uv, depth, normal, bias);
    let l = abs(neighbor_normal_edge_indicator(left_coord, depth, normal, bias) - p);
    let r = abs(neighbor_normal_edge_indicator(right_coord, depth, normal, bias) - p);
    let u = abs(neighbor_normal_edge_indicator(up_coord, depth, normal, bias) - p);
    let d = abs(neighbor_normal_edge_indicator(down_coord, depth, normal, bias) - p);

    let m = max(max(max(l,r),u),d);

    var front_color = textureSample(screen_texture, texture_sampler, screen_uv);
    let size_x = 1. / 160.;
    let size_y = 1. / 90.;
    if m > 0.0 {
        if u == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(0., size_y));
        } else if l == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(-size_x, 0.));
        } else if r == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(size_x, 0.));
        } else  if d == m {
            front_color = textureSample(screen_texture, texture_sampler, screen_uv + vec2<f32>(0., -size_y));
        }
    }
    return front_color;
}

fn get_normal(uv: vec2<i32>) -> vec3<f32> {
    let prepass_normal_sample = textureLoad(prepass_normal_texture, uv, 0);
    let prepass_normal = normalize(prepass_normal_sample.xyz  * 2.0 - vec3(1.0));
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

fn neighbor_normal_edge_indicator(pos : vec2<i32>, depth: f32, normal: vec3<f32>, bias: vec3<f32>) -> f32 {
    var depth_diff = get_depth(pos) - depth;
    
    // Edge pixels should yield to faces closer to the bias direction.
    var normal_diff = dot(normal - get_normal(pos), bias);
    var normal_indicator = clamp(smoothstep(-.01, .01, normal_diff), 0.0, 1.0);
    
    // Only the shallower pixel should detect the normal edge.
    var depth_indicator = clamp(sign(depth_diff * .25 + .0025), 0.0, 1.0);

    return distance(normal, get_normal(pos)) * depth_indicator * normal_indicator;
}

fn depth_edge_indicator(uv: vec2<i32>) -> f32 {
    let offset = 1;        
    let left_coord      = vec2(uv + vec2(-offset,       0));
	let right_coord     = vec2(uv + vec2( offset,       0));
	let up_coord        = vec2(uv + vec2( 0,       offset));
	let down_coord      = vec2(uv + vec2( 0,      -offset));

    var depth = get_depth(uv);                           
    // Difference between depth of neighboring pixels and current.                           
    var depth_diff = 0.0;                   

    depth_diff += clamp(depth - get_depth(left_coord), 0.0, 1.0);                           
    depth_diff += clamp(depth - get_depth(right_coord), 0.0, 1.0);                           
    depth_diff += clamp(depth - get_depth(up_coord), 0.0, 1.0);                           
    depth_diff += clamp(depth - get_depth(down_coord), 0.0, 1.0);
    return floor(smoothstep(0.01, 0.02, depth_diff) * 2.) / 2.;
}

fn normal_edge_indicator(uv: vec2<i32>) -> f32 {
    var depth = get_depth(uv);
    var normal = get_normal(uv);

    let offset = 1;        
    let left_coord      = vec2(uv + vec2(-offset,       0));
	let right_coord     = vec2(uv + vec2( offset,       0));
	let up_coord        = vec2(uv + vec2( 0,       offset));
	let down_coord      = vec2(uv + vec2( 0,      -offset));
    
    var indicator = 0.0;

    let normal_edge_bias = vec3(0., -1., 0.); // This should probably be a parameter.
    indicator += neighbor_normal_edge_indicator(left_coord, depth, normal, normal_edge_bias);
    indicator += neighbor_normal_edge_indicator(right_coord, depth, normal, normal_edge_bias);
    indicator += neighbor_normal_edge_indicator(up_coord, depth, normal, normal_edge_bias);
    indicator += neighbor_normal_edge_indicator(down_coord, depth, normal, normal_edge_bias);

    return step(0.1, indicator);
}

fn lum(color: vec4<f32>) -> f32 {
    var weights = vec4(.2126, .7152, .0722, .0);
    return dot(color, weights);
}

fn smoothSign(x: f32, radius: f32) -> f32 {
    return smoothstep(-radius, radius, x) * 2.0 - 1.0;
}
