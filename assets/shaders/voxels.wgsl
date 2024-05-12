#import bevy_pbr::mesh_view_bindings view
#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_core_pipeline::tonemapping tone_mapping
#import voxels::ray_march ray_march

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
}

@fragment
fn fragment(in: MeshVertexOutput, @builtin(front_facing) front_facing: bool) -> FragmentOutput {
    var res = ray_march(in.world_position.xyz, view, front_facing);

    var pbr = pbr_functions::pbr_input_new();
    pbr.frag_coord = vec4(in.position.xy, -res.clip_pos.z, 1.0);
    pbr.world_position = res.world_pos;
    pbr.world_normal = res.world_normal;
    pbr.N = res.world_normal;
    pbr.V = normalize(view.world_position.xyz - res.world_pos.xyz);
    pbr.material.base_color = vec4(0.0, 1.0, 0.0, 1.0);

    var out: FragmentOutput;
    out.color = tone_mapping(pbr_functions::pbr(pbr), view.color_grading);
    out.depth = res.clip_pos.z / res.clip_pos.w;
    return out;
}
