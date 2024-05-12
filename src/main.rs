mod movement;
mod voxels;

use bevy::prelude::*;
use movement::FlycamPlugin;
use voxels::{ChunkLoader, VoxelRenderPlugin};

fn main() {
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
		mesh: meshes.add(shape::Plane::from_size(50.0).into()),
		transform: Transform::from_xyz(8.0, 0.0, 8.0),
		material: std_materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
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
			intensity: 6000.0,
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
}

#[derive(Component)]
struct PrimaryCamera;
