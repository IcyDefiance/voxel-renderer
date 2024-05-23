use bevy::{prelude::*, utils::HashMap};

use crate::math::aabb::UAabb;

#[derive(Debug)]
pub struct Octree {
	quadrant_size: u32,
	/// position of the corner of the octree with the smallest coordinates
	position: IVec3,
	entry: u32,
	data: Vec<OctreeNode>,
	/// (idx, refcount)
	map: HashMap<OctreeNode, (u32, u32)>,
	/// (start_idx inclusive, end_idx exclusive)
	free_ranges: Vec<(u32, u32)>,
}
impl Octree {
	/// `size` must be a power of 2 and greater than or equal to 2
	pub fn new(size: u32) -> Self {
		if size & (size - 1) != 0 {
			panic!("size must be a power of 2");
		}

		let empty_node = OctreeNode::new();

		let mut map = HashMap::default();
		map.insert(empty_node, (0, 1));

		Self {
			quadrant_size: size / 2,
			position: IVec3::ZERO,
			entry: 0,
			data: vec![empty_node],
			map,
			free_ranges: vec![],
		}
	}

	/// Each component of `pos` must be less than the octree size
	///
	/// `value` must be less than 2^31
	pub fn set_voxel(&mut self, pos: UVec3, id: u32) {
		self.voxel_cursor_mut(pos).set_voxel(id);
	}

	/// Each component of `pos` must be less than the octree size
	pub fn voxel_cursor(&self, pos: UVec3) -> VoxelCursor {
		VoxelCursor::new(self, pos, self.quadrant_size)
	}

	/// Each component of `pos` must be less than the octree size
	pub fn voxel_cursor_mut(&mut self, pos: UVec3) -> VoxelCursorMut {
		VoxelCursorMut::new(self, pos, self.quadrant_size)
	}

	pub fn get_position(&self) -> IVec3 {
		self.position
	}

	/// Set the world position of the octree. This will clear any voxels that are no longer in the octree.
	pub fn set_position(&mut self, pos: IVec3) {
		let pos_diff = pos - self.position;
		let tree_size = self.quadrant_size * 2;

		// figure out the areas that need to be cleared
		let (start_x, size_x) = if pos_diff.x > 0 {
			(0, pos_diff.x as u32)
		} else {
			((tree_size as i32 - pos_diff.x) as u32, (-pos_diff.x) as u32)
		};
		let (start_y, size_y) = if pos_diff.y > 0 {
			(0, pos_diff.y as u32)
		} else {
			((tree_size as i32 - pos_diff.y) as u32, (-pos_diff.y) as u32)
		};
		let (start_z, size_z) = if pos_diff.z > 0 {
			(0, pos_diff.z as u32)
		} else {
			((tree_size as i32 - pos_diff.z) as u32, (-pos_diff.z) as u32)
		};

		// clear for X movement
		self.clear_area(
			UVec3::new(start_x, 0, 0),
			UVec3::new(size_x, tree_size as _, tree_size as _),
		);

		// avoid clearing the corners twice
		let (start_x, size_x) = if start_x == 0 {
			(size_x, tree_size - size_x)
		} else {
			(0, start_x)
		};
		// clear for Y movement
		self.clear_area(
			UVec3::new(start_x, start_y, 0),
			UVec3::new(size_x, size_y, tree_size as _),
		);

		// avoid clearing the corners twice
		let (start_y, size_y) = if start_y == 0 {
			(size_y, tree_size - size_y)
		} else {
			(0, start_y)
		};
		// clear for Z movement
		self.clear_area(
			UVec3::new(start_x, start_y, start_z),
			UVec3::new(size_x, size_y, size_z),
		);
	}

	fn clear_area(&mut self, pos: UVec3, size: UVec3) {
		let area_end = pos + size;

		let mut cursor = self.voxel_cursor_mut(pos);
		while cursor.pos().x < area_end.x {
			while cursor.pos().y < area_end.y {
				while cursor.pos().z < area_end.z {
					cursor.set_voxel(0);
					cursor.move_by(IVec3::new(0, 0, 1));
				}
				cursor.move_by(IVec3::new(0, 1, -(size.z as i32)));
			}
			cursor.move_by(IVec3::new(1, -(size.y as i32), 0));
		}
	}

	fn set_value(&mut self, data_idx: u32, quadrant: UVec3, value: OctreeValue) -> u32 {
		// copy the existing node and update the value
		let mut new_node = self.data[data_idx as usize];
		new_node.set_value(quadrant, value);

		let new_data_idx = self.get_or_insert_node(new_node);

		self.free_node(data_idx);

		new_data_idx
	}

	fn get_or_insert_node(&mut self, node: OctreeNode) -> u32 {
		// reuse another node if possible, otherwise add a new one
		let (new_data_idx, refcount) = self.map.entry(node).or_insert_with(|| {
			if self.free_ranges.len() > 0 {
				// take free node
				let free_range = &mut self.free_ranges[0];
				let new_data_idx = free_range.0;

				self.data[new_data_idx as usize] = node;

				free_range.0 += 1;
				if free_range.0 == free_range.1 {
					self.free_ranges.remove(0);
				}

				(new_data_idx, 0)
			} else {
				// add new node
				let new_data_idx = self.data.len() as u32;
				self.data.push(node);
				(new_data_idx, 0)
			}
		});
		*refcount += 1;
		*new_data_idx
	}

	fn free_node(&mut self, data_idx: u32) {
		let (_, refcount) = self.map.get_mut(&self.data[data_idx as usize]).unwrap();
		*refcount -= 1;

		if *refcount == 0 {
			// this will always fail, because start_idx must already be free, but it'll tell us where to insert this node
			let search_idx = self
				.free_ranges
				.binary_search_by_key(&data_idx, |&(start_idx, _)| start_idx)
				.unwrap_err();

			if search_idx > 0 {
				let (_, prev_end_idx) = self.free_ranges[search_idx - 1];
				if prev_end_idx == data_idx {
					// adjust previous range
					self.free_ranges[search_idx - 1].1 += 1;

					if search_idx < self.free_ranges.len() {
						let (next_start_idx, _) = self.free_ranges[search_idx];
						if next_start_idx == data_idx + 1 {
							// merge with next range
							self.free_ranges[search_idx - 1].1 = self.free_ranges[search_idx].1;
							self.free_ranges.remove(search_idx);
						}
					}

					return;
				}
			} else if search_idx < self.free_ranges.len() {
				let (next_start_idx, _) = self.free_ranges[search_idx];
				if next_start_idx == data_idx + 1 {
					// adjust next range
					self.free_ranges[search_idx].0 -= 1;
					return;
				}
			}

			// insert new range
			self.free_ranges.insert(search_idx, (data_idx, data_idx + 1));

			// remove from map
			self.map.remove(&self.data[data_idx as usize]);
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
struct OctreeNode {
	data: [[[OctreeValue; 2]; 2]; 2],
}
impl OctreeNode {
	fn new() -> Self {
		Self {
			data: Default::default(),
		}
	}

	fn with_value(value: OctreeValue) -> Self {
		Self {
			data: [[[value; 2]; 2]; 2],
		}
	}

	/// x, y, and z must be 0 or 1
	fn value(&self, quadrant: UVec3) -> OctreeValue {
		self.data[quadrant.z as usize][quadrant.y as usize][quadrant.x as usize]
	}

	/// x, y, and z must be 0 or 1
	fn set_value(&mut self, quadrant: UVec3, value: OctreeValue) {
		self.data[quadrant.z as usize][quadrant.y as usize][quadrant.x as usize] = value;
	}
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct OctreeValue(u32);
impl OctreeValue {
	fn new_leaf(value: u32) -> Self {
		Self(value << 1)
	}

	fn new_pointer(value: u32) -> Self {
		Self((value << 1) | 1)
	}

	pub fn is_voxel(&self) -> bool {
		(self.0 & 1) == 0
	}

	pub fn is_pointer(&self) -> bool {
		(self.0 & 1) == 1
	}

	pub fn voxel_id(&self) -> Option<u32> {
		if self.is_voxel() {
			Some(self.to_u32())
		} else {
			None
		}
	}

	pub fn pointer_idx(&self) -> Option<u32> {
		if self.is_pointer() {
			Some(self.to_u32())
		} else {
			None
		}
	}

	fn to_u32(&self) -> u32 {
		self.0 >> 1
	}
}

#[derive(Debug)]
pub struct VoxelCursor<'a> {
	octree: &'a Octree,
	inner: VoxelCursorInner,
}
impl<'a> VoxelCursor<'a> {
	fn new(octree: &'a Octree, pos: UVec3, quadrant_size: u32) -> Self {
		Self {
			octree,
			inner: VoxelCursorInner::new(octree.entry, pos, quadrant_size),
		}
	}

	pub fn is_leaf(&self) -> bool {
		self.inner.is_leaf(&self.octree.data)
	}

	pub fn value(&self) -> OctreeValue {
		self.inner.value(&self.octree.data)
	}

	pub fn move_to_leaf(&mut self) -> &mut Self {
		self.inner.move_to_leaf(&self.octree.data);
		self
	}

	pub fn move_by(&mut self, pos: IVec3) {
		self.inner.move_by(pos);
	}

	pub fn pos(&self) -> UVec3 {
		self.inner.pos
	}

	pub fn quadrant_size(&self) -> u32 {
		self.inner.quadrant_size
	}
}

pub struct VoxelCursorMut<'a> {
	octree: &'a mut Octree,
	inner: VoxelCursorInner,
}
impl<'a> VoxelCursorMut<'a> {
	fn new(octree: &'a mut Octree, pos: UVec3, quadrant_size: u32) -> Self {
		let data_index = octree.entry;
		Self {
			octree,
			inner: VoxelCursorInner::new(data_index, pos, quadrant_size),
		}
	}

	pub fn is_leaf(&self) -> bool {
		self.inner.is_leaf(&self.octree.data)
	}

	pub fn value(&self) -> OctreeValue {
		self.inner.value(&self.octree.data)
	}

	pub fn set_voxel(&mut self, id: u32) {
		self.move_to_leaf();

		if id == self.value().to_u32() {
			return;
		}

		while self.quadrant_size() > 1 {
			self.split_quadrant();
		}

		self.set_value(OctreeValue::new_leaf(id));

		let mut child_idx = self.inner.data_idx;
		let mut parent_quadrant_size = self.quadrant_size() * 2;
		for parent_idx in self.inner.parent_idxs.iter_mut().rev() {
			let node_pos = self.inner.pos & (parent_quadrant_size * 2 - 1);
			let quadrant = get_quadrant(node_pos, parent_quadrant_size);

			*parent_idx = self
				.octree
				.set_value(*parent_idx, quadrant, OctreeValue::new_pointer(child_idx));
			child_idx = *parent_idx;
			parent_quadrant_size *= 2;
		}
		if self.inner.parent_idxs.len() > 0 {
			self.octree.entry = self.inner.parent_idxs[0];
		}
	}

	pub fn move_to_leaf(&mut self) {
		self.inner.move_to_leaf(&self.octree.data);
	}

	pub fn move_by(&mut self, pos: IVec3) {
		self.inner.move_by(pos);
	}

	pub fn pos(&self) -> UVec3 {
		self.inner.pos
	}

	pub fn quadrant_size(&self) -> u32 {
		self.inner.quadrant_size
	}

	/// x, y, and z must be 0 or 1
	fn split_quadrant(&mut self) {
		if self.inner.quadrant_size == 1 {
			panic!("Can't split a quadrant of size 1");
		}
		if !self.is_leaf() {
			panic!("Quadrant already split");
		}

		let new_node = OctreeNode::with_value(self.value());
		let new_data_idx = self.octree.get_or_insert_node(new_node);

		self.set_value(OctreeValue::new_pointer(new_data_idx));
		if self.inner.parent_idxs.len() == 0 {
			self.octree.entry = self.inner.data_idx;
		}

		self.inner.move_to_child_idx(new_data_idx);
	}

	fn set_value(&mut self, value: OctreeValue) {
		self.inner.data_idx = self
			.octree
			.set_value(self.inner.data_idx, self.inner.get_quadrant(), value);
	}
}

#[derive(Debug)]
struct VoxelCursorInner {
	/// we keep around the parent indexes to make traversal faster
	/// we might want to replace this with a stack allocated array, once we can make a good benchmark compare the two,
	/// but we'd need a compile-time max size for an octree
	parent_idxs: Vec<u32>,
	data_idx: u32,
	pos: UVec3,
	quadrant_size: u32,
}
impl VoxelCursorInner {
	fn new(data_idx: u32, pos: UVec3, quadrant_size: u32) -> Self {
		Self {
			parent_idxs: Vec::with_capacity(quadrant_size.trailing_zeros() as usize),
			data_idx,
			pos,
			quadrant_size,
		}
	}

	fn move_by(&mut self, pos: IVec3) {
		let subtree_box = self.subtree_box();

		// TODO: replace with wrapping_add_signed once glam is updated
		self.pos = (self.pos.as_ivec3() + pos).as_uvec3();

		while !subtree_box.contains(self.pos) {
			self.move_to_parent();
		}
	}

	fn is_leaf(&self, data: &[OctreeNode]) -> bool {
		self.value(data).is_voxel()
	}

	fn move_to_leaf(&mut self, data: &[OctreeNode]) {
		loop {
			let value = self.value(data);
			if let Some(next_idx) = value.pointer_idx() {
				self.move_to_child_idx(next_idx);
			} else {
				break;
			}
		}
	}

	fn value(&self, data: &[OctreeNode]) -> OctreeValue {
		data[self.data_idx as usize].value(self.get_quadrant())
	}

	fn get_quadrant(&self) -> UVec3 {
		// similar to `self.pos %= self.quadrant_size` but hopefully faster. need to benchmark
		// this works as long as quadrant_size is a power of 2
		let node_pos = self.pos & (self.quadrant_size * 2 - 1);
		get_quadrant(node_pos, self.quadrant_size)
	}

	fn move_to_child_idx(&mut self, idx: u32) {
		self.parent_idxs.push(self.data_idx);
		self.data_idx = idx;
		self.quadrant_size /= 2;
	}

	fn move_to_parent(&mut self) {
		self.quadrant_size *= 2;
		self.data_idx = self.parent_idxs.pop().unwrap();
	}

	fn subtree_box(&self) -> UAabb {
		let subtree_start = self.pos & !(self.quadrant_size * 2 - 1);
		let subtree_end = subtree_start + self.quadrant_size * 2;
		UAabb::new(subtree_start, subtree_end)
	}
}

/// each component of `pos` must be less than quadrant_size * 2
fn get_quadrant(pos: UVec3, quadrant_size: u32) -> UVec3 {
	// each component of quadrant should be 0 if the coord is less than quadrant_size, 1 otherwise
	// this works as long as each component of pos is less than quadrant_size * 2
	pos >> quadrant_size.trailing_zeros()
}
