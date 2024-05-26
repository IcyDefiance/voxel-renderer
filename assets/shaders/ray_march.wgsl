#define_import_path voxels::ray_march

#import bevy_pbr::mesh_functions::get_model_matrix
#import bevy_render::view::View
#import bevy_pbr::mesh_bindings::mesh

@group(2) @binding(0)
var chunk_texture: texture_3d<f32>;

struct RayMarchOutput {
    world_pos: vec4<f32>,
    clip_pos: vec4<f32>,
    world_normal: vec3<f32>,
}

// Based on A Fast Voxel Traversal Algorithm for Ray Tracing (http://www.cse.yorku.ca/~amana/research/grid.pdf) with
// more concise code taken from Branchless Voxel Raycasting (https://www.shadertoy.com/view/4dX3zl)
fn ray_march(instance_index: u32, in_world_position: vec3<f32>, view: View, front_facing: bool) -> RayMarchOutput {
    let forward = normalize(in_world_position - view.world_position);
    let step = sign(forward);

    let chunk_origin = get_model_matrix(instance_index)[3].xyz;
    var chunk_pos = vec3(0.0);
    if (front_facing) {
      chunk_pos = in_world_position - chunk_origin;
    } else {
      chunk_pos = view.world_position - chunk_origin;
    }

    let t_delta = abs(1.0 / forward);

    var voxel_idx: vec3<f32> = min(floor(chunk_pos), vec3(15.0));
    var pos_in_voxel = chunk_pos - voxel_idx;
    var t_max = (step * (chunk_origin + voxel_idx - view.world_position) + (step * 0.5) + 0.5) * t_delta;
    var world_normal = vec3<f32>(vec3(16.0) == chunk_pos) - vec3<f32>(vec3(0.0) == chunk_pos);

    while (true) {
        let voxel = textureLoad(chunk_texture, vec3<i32>(voxel_idx), 0);
        if (voxel.x > 0.0) {
            let voxel_world_pos = chunk_origin + voxel_idx;
            let world_pos = vec4(intersect_ray_aabb(view.world_position, forward, voxel_world_pos, voxel_world_pos + 1.0), 1.0);
            let clip_pos = view.view_proj * world_pos;

            var out: RayMarchOutput;
            out.world_pos = world_pos;
            out.clip_pos = clip_pos;
            out.world_normal = world_normal;
            return out;
        }

        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        t_max += mask * t_delta;
        voxel_idx += mask * step;
        world_normal = -(mask * step);
        if (
            voxel_idx.x < 0.0 || voxel_idx.x >= 16.0
            || voxel_idx.y < 0.0 || voxel_idx.y >= 16.0
            || voxel_idx.z < 0.0 || voxel_idx.z >= 16.0
        ) {
            discard;
        }

        // if (t_max.x < t_max.y) {
        //     if (t_max.x < t_max.z) {
        //         voxel_idx.x += step.x;
        //         t_max.x += t_delta.x;
        //         world_normal = vec3<f32>(-step.x, 0.0, 0.0);
        //         if (voxel_idx.x < 0.0 || voxel_idx.x >= 16.0) {
        //             discard;
        //         }
        //     } else {
        //         voxel_idx.z += step.z;
        //         t_max.z += t_delta.z;
        //         world_normal = vec3<f32>(0.0, 0.0, -step.z);
        //         if (voxel_idx.z < 0.0 || voxel_idx.z >= 16.0) {
        //             discard;
        //         }
        //     }
        // } else {
        //     if (t_max.y < t_max.z) {
        //         voxel_idx.y += step.y;
        //         t_max.y += t_delta.y;
        //         world_normal = vec3<f32>(0.0, -step.y, 0.0);
        //         if (voxel_idx.y < 0.0 || voxel_idx.y >= 16.0) {
        //             discard;
        //         }
        //     } else {
        //         voxel_idx.z += step.z;
        //         t_max.z += t_delta.z;
        //         world_normal = vec3<f32>(0.0, 0.0, -step.z);
        //         if (voxel_idx.z < 0.0 || voxel_idx.z >= 16.0) {
        //             discard;
        //         }
        //     }
        // }
    }
    
    var out: RayMarchOutput;
    out.world_pos = vec4(0.0);
    out.clip_pos = vec4(0.0);
    out.world_normal = vec3(0.0);
    return out;
}

fn intersect_ray_aabb(ray_origin: vec3<f32>, ray_dir: vec3<f32>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> vec3<f32> {
    let inv_dir = 1.0 / ray_dir;
    let t_min = (aabb_min - ray_origin) * inv_dir;
    let t_max = (aabb_max - ray_origin) * inv_dir;

    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);
    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);

    let intersection = ray_origin + ray_dir * t_near;
    let no_intersection = vec3<f32>(-1.0);
    let result = mix(no_intersection, intersection, f32(t_near <= t_far && t_near >= 0.0));

    return result;
}
