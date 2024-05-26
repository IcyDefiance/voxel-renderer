mod movement;

use bevy::prelude::*;
use bevy_voxels::{octree::Octree, ChunkLoader, VoxelRenderPlugin};
use movement::FlycamPlugin;
use std::env::set_var;

fn main() {
	set_var("WGPU_BACKEND", "vulkan");

	App::new()
		.add_plugins((DefaultPlugins, FlycamPlugin, VoxelRenderPlugin))
		.add_systems(Startup, setup)
		.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
	// plane
	commands.spawn(PbrBundle {
		mesh: meshes.add(Plane3d::default().mesh().size(50., 50.)),
		transform: Transform::from_xyz(8.0, 0.0, 8.0),
		material: std_materials.add(Color::rgb(1.0, 1.0, 1.0)),
		..default()
	});

	// camera
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(-10.0, 20.0, 25.0).looking_at(Vec3::new(8.0, 8.0, 8.0), Vec3::Y),
			..default()
		},
		PrimaryCamera,
		ChunkLoader::radius(8),
	));

	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 6000000.0,
			range: 200.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(-5.0, 30.0, 15.0),
		..default()
	});

	// light
	// commands.spawn(DirectionalLightBundle {
	// 	directional_light: DirectionalLight {
	// 		illuminance: 10000.0,
	// 		shadows_enabled: true,
	// 		..default()
	// 	},
	// 	transform: Transform::from_xyz(-5.0, 30.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
	// 	..default()
	// });

	let mut octree = Octree::new(16);
	octree.set_voxel(UVec3 { x: 8, y: 8, z: 8 }, 1);
	info!("{:?}", octree);
	let value = octree.voxel_cursor(UVec3 { x: 8, y: 8, z: 8 }).move_to_leaf().value();
	info!("{:?}", value);
	let value = octree.voxel_cursor(UVec3 { x: 9, y: 8, z: 8 }).move_to_leaf().value();
	info!("{:?}", value);
}

#[derive(Component)]
struct PrimaryCamera;
