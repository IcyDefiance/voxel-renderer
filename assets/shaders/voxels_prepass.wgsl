#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::utils::coords_to_viewport_uv
#import voxels::ray_march::ray_march
#import bevy_pbr::prepass_io::VertexOutput

struct FragmentOutput {
#ifdef NORMAL_PREPASS
    @location(0) normal: vec4<f32>,
#endif

#ifdef MOTION_VECTOR_PREPASS
    @location(1) motion_vector: vec2<f32>,
#endif

#ifdef DEFERRED_PREPASS
    @location(2) deferred: vec4<u32>,
    @location(3) deferred_lighting_pass_id: u32,
#endif

    @builtin(frag_depth) frag_depth: f32,
}

@group(1) @binding(0)
var chunk_texture: texture_3d<f32>;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front_facing: bool) -> FragmentOutput {
    let viewport_uv = coords_to_viewport_uv(in.position.xy, view.viewport);
    let viewport_ndc = viewport_uv * 2.0 - 1.0;
    let clip_pos = vec4(viewport_ndc.x, -viewport_ndc.y, in.position.z, 1.0);
    let world_pos = view.inverse_view_proj * clip_pos;
    var res = ray_march(in.instance_index, (world_pos / world_pos.w).xyz, view, front_facing);

    var out: FragmentOutput;
#ifdef NORMAL_PREPASS
    out.normal = vec4(res.world_normal, 1.0);
#endif // NORMAL_PREPASS
    // out.frag_depth = 0.05;
    out.frag_depth = res.clip_pos.z / res.clip_pos.w;
    return out;
}
