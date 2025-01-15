use godot::prelude::*;

#[derive(Hash, Eq, PartialEq)]
pub struct Vector3Hash {
    x: i32,
    y: i32,
    z: i32,
}

impl Vector3Hash {
    pub fn from_vector3(v: Vector3) -> Self {
        Self {
            x: (v.x * 100000.0) as i32,
            y: (v.y * 100000.0) as i32,
            z: (v.z * 100000.0) as i32,
        }
    }
}
