#import bevy_pbr::prepass_bindings view
#import bevy_pbr::utils coords_to_viewport_uv
#import voxels::ray_march ray_march

@group(1) @binding(0)
var chunk_texture: texture_3d<f32>;

struct FragmentInput {
    @builtin(position) position: vec4<f32>,
    @builtin(front_facing) front_facing: bool,
    
#ifdef VERTEX_UVS
    @location(0) uv: vec2<f32>,
#endif // VERTEX_UVS

#ifdef NORMAL_PREPASS
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(2) world_tangent: vec4<f32>,
#endif // VERTEX_TANGENTS
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
    @location(3) world_position: vec4<f32>,
    @location(4) previous_world_position: vec4<f32>,
#endif // MOTION_VECTOR_PREPASS

#ifdef DEPTH_CLAMP_ORTHO
    @location(5) clip_position_unclamped: vec4<f32>,
#endif // DEPTH_CLAMP_ORTHO
};

struct FragmentOutput {
#ifdef NORMAL_PREPASS
    @location(0) normal: vec4<f32>,
#endif // NORMAL_PREPASS

    @builtin(frag_depth) frag_depth: f32,
}

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    let viewport_uv = coords_to_viewport_uv(in.position.xy, view.viewport);
    let viewport_ndc = viewport_uv * 2.0 - 1.0;
    let clip_pos = vec4(viewport_ndc.x, -viewport_ndc.y, in.position.z, 1.0);
    let world_pos = view.inverse_view_proj * clip_pos;
    var res = ray_march((world_pos / world_pos.w).xyz, view, in.front_facing);

    var out: FragmentOutput;
#ifdef NORMAL_PREPASS
    out.normal = vec4(res.world_normal, 1.0);
#endif // NORMAL_PREPASS
    // out.frag_depth = 0.05;
    out.frag_depth = res.clip_pos.z / res.clip_pos.w;
    return out;
}
