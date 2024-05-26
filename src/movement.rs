use bevy::{
	input::mouse::MouseMotion,
	prelude::*,
	window::{CursorGrabMode, WindowMode},
};

use crate::PrimaryCamera;

pub struct FlycamPlugin;
impl Plugin for FlycamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (window_controls, flycam));
	}
}

fn window_controls(
	mut commands: Commands,
	mut window: Query<(Entity, &mut Window)>,
	mouse: Res<ButtonInput<MouseButton>>,
	key: Res<ButtonInput<KeyCode>>,
) {
	if mouse.just_pressed(MouseButton::Left) {
		let (_, mut window) = window.single_mut();
		window.cursor.visible = false;
		window.cursor.grab_mode = CursorGrabMode::Locked;
	}

	if key.just_pressed(KeyCode::Escape) {
		let (window_entity, mut window) = window.single_mut();
		if window.cursor.grab_mode != CursorGrabMode::None {
			window.cursor.visible = true;
			window.cursor.grab_mode = CursorGrabMode::None;
		} else {
			commands.entity(window_entity).despawn();
		}
	}

	if key.just_pressed(KeyCode::F11) {
		let (_, mut window) = window.single_mut();
		if window.mode == WindowMode::BorderlessFullscreen {
			window.mode = WindowMode::Windowed;
		} else {
			window.mode = WindowMode::BorderlessFullscreen;
		}
	}
}

fn flycam(
	mut cameras: Query<&mut Transform, With<PrimaryCamera>>,
	time: Res<Time>,
	window: Query<&Window>,
	mut mouse_motion_events: EventReader<MouseMotion>,
	key: Res<ButtonInput<KeyCode>>,
) {
	if window.single().cursor.grab_mode != CursorGrabMode::Locked {
		return;
	}

	let mut camera_transform = cameras.single_mut();

	for event in mouse_motion_events.read() {
		camera_transform.rotate_axis(Vec3::Y, -event.delta.x * 0.003);
		camera_transform.rotate_local_axis(Vec3::X, -event.delta.y * 0.003);
	}

	let forward = camera_transform.forward();
	let right = camera_transform.right();
	let up = camera_transform.up();
	if key.pressed(KeyCode::KeyW) {
		camera_transform.translation += forward * time.delta_seconds() * 10.0;
	}
	if key.pressed(KeyCode::KeyS) {
		camera_transform.translation -= forward * time.delta_seconds() * 10.0;
	}
	if key.pressed(KeyCode::KeyA) {
		camera_transform.translation -= right * time.delta_seconds() * 10.0;
	}
	if key.pressed(KeyCode::KeyD) {
		camera_transform.translation += right * time.delta_seconds() * 10.0;
	}
	if key.pressed(KeyCode::Space) {
		camera_transform.translation += up * time.delta_seconds() * 10.0;
	}
	if key.pressed(KeyCode::ShiftLeft) {
		camera_transform.translation -= up * time.delta_seconds() * 10.0;
	}
}
