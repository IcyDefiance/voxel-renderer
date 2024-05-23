pub mod octree;

mod math;

use bevy::{
	pbr::{MaterialPipeline, MaterialPipelineKey},
	prelude::*,
	reflect::{TypePath, TypeUuid},
	render::{
		mesh::MeshVertexBufferLayout,
		render_resource::{
			AddressMode, AsBindGroup, Extent3d, FilterMode, RenderPipelineDescriptor, SamplerDescriptor, ShaderRef,
			SpecializedMeshPipelineError, TextureDimension, TextureFormat,
		},
		texture::ImageSampler,
	},
	utils::HashMap,
};
use std::sync::{Arc, Weak};

pub struct VoxelRenderPlugin;
impl Plugin for VoxelRenderPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
			.add_systems(Startup, setup)
			.add_systems(Update, load_chunks);
	}
}

#[derive(Component)]
pub struct ChunkLoader {
	radius: i32,
	loaded: HashMap<IVec3, Arc<Entity>>,
}
impl ChunkLoader {
	pub fn radius(radius: i32) -> Self {
		Self {
			radius,
			loaded: HashMap::new(),
		}
	}
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, assets: Res<AssetServer>) {
	commands.insert_resource(VoxelRenderGlobals {
		ray_march_shader: assets.load::<Shader, _>("shaders/ray_march.wgsl"),
	});
	commands.insert_resource(LoadedChunks(HashMap::new()));
	commands.insert_resource(ChunkBox::new(&mut meshes));
}

fn load_chunks(
	mut commands: Commands,
	mut loaded_chunks: ResMut<LoadedChunks>,
	mut materials: ResMut<Assets<ChunkMaterial>>,
	mut images: ResMut<Assets<Image>>,
	chunk_box: Res<ChunkBox>,
	mut loaders: Query<(&mut ChunkLoader, &Transform)>,
) {
	for (mut loader, transform) in loaders.iter_mut() {
		let chunk_position = (transform.translation / 16.0).as_ivec3();
		let radius3 = IVec3::new(loader.radius, loader.radius, loader.radius);
		let load_start = chunk_position - radius3;
		let load_end = chunk_position + radius3;

		// unload chunks outside of the radius, if no other loader is loading them
		let mut to_remove = vec![];
		for &chunk_position in loader.loaded.keys() {
			if chunk_position.x < load_start.x
				|| chunk_position.x > load_end.x
				|| chunk_position.y < load_start.y
				|| chunk_position.y > load_end.y
				|| chunk_position.z < load_start.z
				|| chunk_position.z > load_end.z
			{
				to_remove.push(chunk_position);
			}
		}
		for chunk_position in to_remove {
			let entity = loader.loaded.remove(&chunk_position).unwrap();
			if let Some(entity) = Arc::into_inner(entity) {
				commands.entity(entity).despawn();
				loaded_chunks.remove(&chunk_position);
			}
		}

		// load chunks inside the radius, if not already loaded
		for x in load_start.x..=load_end.x {
			for y in load_start.y..=load_end.y {
				for z in load_start.z..=load_end.z {
					let chunk_position = IVec3::new(x, y, z);
					if loader.loaded.contains_key(&chunk_position) {
						continue;
					}

					let chunk_entity = if let Some(entity) = loaded_chunks.get(&chunk_position) {
						entity.upgrade().unwrap()
					} else {
						let entity = commands.spawn((
							MaterialMeshBundle {
								mesh: chunk_box.mesh.clone(),
								transform: Transform::from_xyz(
									chunk_position.x as f32 * 16.0,
									chunk_position.y as f32 * 16.0,
									chunk_position.z as f32 * 16.0,
								),
								material: materials.add(ChunkMaterial::new(&mut images)),
								..default()
							},
							Chunk,
						));
						let entity = Arc::new(entity.id());
						loaded_chunks.insert(chunk_position, Arc::downgrade(&entity));
						entity
					};
					loader.loaded.insert(chunk_position, chunk_entity);
				}
			}
		}
	}
}

#[derive(Component)]
struct Chunk;

#[derive(Deref, DerefMut, Resource)]
struct LoadedChunks(HashMap<IVec3, Weak<Entity>>);

#[derive(Resource)]
struct ChunkBox {
	mesh: Handle<Mesh>,
}
impl ChunkBox {
	fn new(meshes: &mut Assets<Mesh>) -> Self {
		let mesh = meshes.add(
			shape::Box {
				min_x: 0.,
				max_x: 16.,
				min_y: 0.,
				max_y: 16.,
				min_z: 0.,
				max_z: 16.,
			}
			.into(),
		);
		Self { mesh }
	}
}

#[derive(Resource)]
struct VoxelRenderGlobals {
	#[allow(dead_code)]
	ray_march_shader: Handle<Shader>,
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
struct ChunkMaterial {
	#[texture(0, dimension = "3d")]
	chunk: Handle<Image>,
}
impl ChunkMaterial {
	fn new(images: &mut Assets<Image>) -> Self {
		Self {
			chunk: init_chunk(images),
		}
	}
}
impl Material for ChunkMaterial {
	fn prepass_fragment_shader() -> ShaderRef {
		"shaders/voxels_prepass.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/voxels.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn specialize(
		_pipeline: &MaterialPipeline<Self>,
		descriptor: &mut RenderPipelineDescriptor,
		_layout: &MeshVertexBufferLayout,
		_key: MaterialPipelineKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		descriptor.primitive.cull_mode = None;

		Ok(())
	}
}

pub fn init_chunk(images: &mut Assets<Image>) -> Handle<Image> {
	let mut data = vec![0; 16 * 16 * 16 * 4];

	// set the middle voxel to 1
	set_voxel(&mut data, 8, 8, 8, 1);
	// set another voxel to 1
	set_voxel(&mut data, 10, 8, 8, 1);
	// set another voxel to 1
	set_voxel(&mut data, 8, 10, 8, 1);
	// set another voxel to 1
	set_voxel(&mut data, 9, 10, 8, 1);
	// set corner voxel to 1
	set_voxel(&mut data, 0, 0, 0, 1);

	let size = Extent3d {
		width: 16,
		height: 16,
		depth_or_array_layers: 16,
	};
	let format = TextureFormat::Rgba8UnormSrgb;
	let mut image = Image::new(size, TextureDimension::D3, data, format);

	let sampler = SamplerDescriptor {
		address_mode_u: AddressMode::ClampToEdge,
		address_mode_v: AddressMode::ClampToEdge,
		address_mode_w: AddressMode::ClampToEdge,
		mag_filter: FilterMode::Nearest,
		min_filter: FilterMode::Nearest,
		mipmap_filter: FilterMode::Nearest,
		..Default::default()
	};
	image.sampler_descriptor = ImageSampler::Descriptor(sampler);

	images.add(image)
}

fn set_voxel(voxels: &mut Vec<u8>, x: usize, y: usize, z: usize, value: u8) {
	let index = (z * 16 * 16 + y * 16 + x) * 4;
	voxels[index + 0] = value;
	voxels[index + 1] = value;
	voxels[index + 2] = value;
	voxels[index + 3] = value;
}
