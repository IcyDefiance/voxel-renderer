use bevy::math::UVec3;

pub struct UAabb {
	pub min: UVec3,
	pub max: UVec3,
}
impl UAabb {
	pub fn new(min: UVec3, max: UVec3) -> Self {
		Self { min, max }
	}

	pub fn contains(&self, point: UVec3) -> bool {
		point.x >= self.min.x
			&& point.x <= self.max.x
			&& point.y >= self.min.y
			&& point.y <= self.max.y
			&& point.z >= self.min.z
			&& point.z <= self.max.z
	}
}
