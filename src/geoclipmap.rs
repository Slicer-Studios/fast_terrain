use godot::{classes::{rendering_server::{ArrayType, PrimitiveType}, RenderingServer}, meta::ParamType, prelude::*};
use std::collections::HashMap;
use crate::types::Vector3Hash;

pub struct GeoClipMap;

impl GeoClipMap {
    fn subdivide_half(vertices: &mut PackedVector3Array, indices: &mut PackedInt32Array) {
        let mut new_vertices = PackedVector3Array::new();
        let mut new_indices = PackedInt32Array::new();
        let mut vertex_map = HashMap::new();

        let midpoint = |p1: Vector3, p2: Vector3| -> Vector3 {
            (p1 + p2) / 2.0
        };

        let find_or_add_vertex = |vertex_map: &mut HashMap<Vector3Hash, i32>, 
                                 new_vertices: &mut PackedVector3Array,
                                 vertex: Vector3| -> i32 {
            let key = Vector3Hash::from_vector3(vertex);
            if let Some(&index) = vertex_map.get(&key) {
                index
            } else {
                let index = new_vertices.len() as i32;
                vertex_map.insert(key, index);
                new_vertices.push(vertex);
                index
            }
        };

        let indices_vec: Vec<i32> = indices.to_vec();
        for chunk in indices_vec.chunks(3) {
            let id_0 = chunk[0];
            let id_1 = chunk[1];
            let id_2 = chunk[2];

            let a = vertices.get(id_0 as usize).unwrap();
            let b = vertices.get(id_1 as usize).unwrap();
            let c = vertices.get(id_2 as usize).unwrap();

            let length_ab = (b - a).length_squared();
            let length_bc = (c - b).length_squared();
            let length_ca = (a - c).length_squared();

            if length_ab >= length_bc && length_ab >= length_ca {
                let a_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, a);
                let b_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, b);
                let c_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, c);
                let mid_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, midpoint(a, b));

                new_indices.push(a_id);
                new_indices.push(mid_id);
                new_indices.push(c_id);
                new_indices.push(mid_id);
                new_indices.push(b_id);
                new_indices.push(c_id);
            } else if length_bc >= length_ab && length_bc >= length_ca {
                let a_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, a);
                let b_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, b);
                let c_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, c);
                let mid_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, midpoint(b, c));

                new_indices.push(b_id);
                new_indices.push(mid_id);
                new_indices.push(a_id);
                new_indices.push(mid_id);
                new_indices.push(c_id);
                new_indices.push(a_id);
            } else {
                let a_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, a);
                let b_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, b);
                let c_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, c);
                let mid_id = find_or_add_vertex(&mut vertex_map, &mut new_vertices, midpoint(c, a));

                new_indices.push(c_id);
                new_indices.push(mid_id);
                new_indices.push(b_id);
                new_indices.push(mid_id);
                new_indices.push(a_id);
                new_indices.push(b_id);
            }
        }

        vertices.clear();
        vertices.extend_array(&new_vertices);
        indices.clear();
        indices.extend_array(&new_indices);
    }

    fn create_mesh(vertices: &PackedVector3Array, indices: &PackedInt32Array, aabb: Aabb) -> Rid {
        let mut arrays = Array::new();
        arrays.resize(ArrayType::MAX.ord() as usize, Variant::nil().owned_to_arg());
        
        arrays.set(ArrayType::VERTEX.ord() as usize, vertices.to_variant().owned_to_arg());
        arrays.set(ArrayType::INDEX.ord() as usize, indices.to_variant().owned_to_arg());

        let mut normals = PackedVector3Array::new();
        normals.resize(vertices.len());
        normals.fill(Vector3::new(0.0, 1.0, 0.0));
        arrays.set(ArrayType::NORMAL.ord() as usize, normals.to_variant().owned_to_arg());

        let mut tangents = PackedFloat32Array::new();
        tangents.resize(vertices.len() * 4);
        tangents.fill(0.0);
        arrays.set(ArrayType::TANGENT.ord() as usize, tangents.to_variant().owned_to_arg());

        let mut rendering_server = RenderingServer::singleton();
        let mesh = rendering_server.mesh_create();
        rendering_server.mesh_add_surface_from_arrays(
            mesh,
            PrimitiveType::TRIANGLES,
            &arrays
        );

        rendering_server.mesh_set_custom_aabb(mesh, aabb);

        mesh
    }

    fn patch_2d(x: i32, y: i32, resolution: i32) -> i32 {
        y * resolution + x
    }

    pub fn generate(size: i32, levels: i32) -> Vec<Rid> {
        godot_print!("Generating meshes of size: {} levels: {}", size, levels);

        let tile_resolution = size;
        let patch_vert_resolution = tile_resolution + 1;
        let clipmap_resolution = tile_resolution * 4 + 1;
        let clipmap_vert_resolution = clipmap_resolution + 1;

        // Tile mesh
        let (tile_mesh, tile_inner_mesh) = {
            let mut vertices = PackedVector3Array::new();
            vertices.resize((patch_vert_resolution * patch_vert_resolution) as usize);
            let mut indices = PackedInt32Array::new();
            indices.resize((tile_resolution * tile_resolution * 6) as usize);

            let mut n = 0;
            for y in 0..patch_vert_resolution {
                for x in 0..patch_vert_resolution {
                    vertices[n] = Vector3::new(x as f32, 0.0, y as f32);
                    n += 1;
                }
            }

            n = 0;
            for y in 0..tile_resolution {
                for x in 0..tile_resolution {
                    indices[n] = Self::patch_2d(x, y, patch_vert_resolution);
                    indices[n + 1] = Self::patch_2d(x + 1, y + 1, patch_vert_resolution);
                    indices[n + 2] = Self::patch_2d(x, y + 1, patch_vert_resolution);
                    indices[n + 3] = Self::patch_2d(x, y, patch_vert_resolution);
                    indices[n + 4] = Self::patch_2d(x + 1, y, patch_vert_resolution);
                    indices[n + 5] = Self::patch_2d(x + 1, y + 1, patch_vert_resolution);
                    n += 6;
                }
            }

            let aabb = Aabb::new(
                Vector3::ZERO,
                Vector3::new(patch_vert_resolution as f32, 0.1, patch_vert_resolution as f32)
            );

            let inner_mesh = Self::create_mesh(&vertices, &indices, aabb);
            Self::subdivide_half(&mut vertices, &mut indices);
            let outer_mesh = Self::create_mesh(&vertices, &indices, aabb);

            (outer_mesh, inner_mesh)
        };

        // Filler mesh
        let (filler_mesh, filler_inner_mesh, aabb) = {
            let mut vertices = PackedVector3Array::new();
            vertices.resize(patch_vert_resolution as usize);
            let mut indices = PackedInt32Array::new();
            indices.resize((tile_resolution * tile_resolution * 6) as usize);

            let mut n = 0;
            for y in 0..patch_vert_resolution {
                for x in 0..patch_vert_resolution {
                    vertices[n] = Vector3::new(x as f32, 0.0, y as f32);
                    n += 1;
                }
            }

            let mut n = 0;
            for y in 0..tile_resolution {
                for x in 0..tile_resolution {
                    indices[n] = Self::patch_2d(x, y, patch_vert_resolution);
                    indices[n + 1] = Self::patch_2d(x + 1, y + 1, patch_vert_resolution);
                    indices[n + 2] = Self::patch_2d(x, y + 1, patch_vert_resolution);
                    indices[n + 3] = Self::patch_2d(x, y, patch_vert_resolution);
                    indices[n + 4] = Self::patch_2d(x + 1, y, patch_vert_resolution);
                    indices[n + 5] = Self::patch_2d(x + 1, y + 1, patch_vert_resolution);
                    n += 6;
                }
            }

            let aabb = Aabb::new(
                Vector3::ZERO,
                Vector3::new(patch_vert_resolution as f32, 0.1, patch_vert_resolution as f32)
            );
            let tile_inner_mesh = Self::create_mesh(&vertices, &indices, aabb);
            GeoClipMap::subdivide_half(&mut vertices, &mut indices);
            let tile_mesh = Self::create_mesh(&vertices, &indices, aabb);
            (tile_mesh, tile_inner_mesh, aabb)
        };

        // Trim mesh
        let (trim_mesh, trim_inner_mesh) = {
            let mut vertices = PackedVector3Array::new();
            vertices.resize((patch_vert_resolution * 8) as usize);
            let mut indices = PackedInt32Array::new();
            indices.resize((tile_resolution * 24) as usize);

            let mut n = 0;
            let offset = tile_resolution;

            for i in 0..patch_vert_resolution {
                vertices[n] = Vector3::new((offset + i + 1) as f32, 0.0, 0.0);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new((offset + i + 1) as f32, 0.0, 1.0);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            for i in 0..patch_vert_resolution {
                vertices[n] = Vector3::new(1.0, 0.0, (offset + i + 1) as f32);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new(0.0, 0.0, (offset + i + 1) as f32);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            for i in 0..patch_vert_resolution {
                vertices[n] = Vector3::new((-offset + i) as f32, 0.0, 1.0);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new((-offset + i) as f32, 0.0, 0.0);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            for i in 0..patch_vert_resolution {
                vertices[n] = Vector3::new(0.0, 0.0, (-offset + i) as f32);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new(1.0, 0.0, (-offset + i) as f32);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            let mut n = 0;
            for i in 0..(tile_resolution * 4)  {
                let arm = i / tile_resolution;
                
                let bl = (arm + i) * 2 + 0;
                let br = (arm + i) * 2 + 1;
                let tl = (arm + i) * 2 + 2;
                let tr = (arm + i) * 2 + 3;

                if arm % 2 == 0 {
                    indices[n] = br;
                    indices[n + 1] = bl;
                    indices[n + 2] = tr;
                    indices[n + 3] = bl;
                    indices[n + 4] = tl;
                    indices[n + 5] = tr;
                    n += 6;
                } else {
                    indices[n] = br;
                    indices[n + 1] = bl;
                    indices[n + 2] = tl;
                    indices[n + 3] = br;
                    indices[n + 4] = tl;
                    indices[n + 5] = tr;
                    n += 6;
                }
            }

            let filler_inner_mesh = Self::create_mesh(&vertices, &indices, aabb);
            Self::subdivide_half(&mut vertices, &mut indices);
            let filler_mesh = Self::create_mesh(&vertices, &indices, aabb);
            (filler_mesh, filler_inner_mesh)
        };

        // Cross mesh
        let cross_mesh = {
            let mut vertices = PackedVector3Array::new();
            vertices.resize((patch_vert_resolution * 8) as usize);
            let mut indices = PackedInt32Array::new();
            indices.resize((tile_resolution * 24 + 6) as usize);

            let mut n = 0;
            for i in 0..(patch_vert_resolution * 2) {
                vertices[n] = Vector3::new((i - tile_resolution) as f32, 0.0, 0.0);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new((i - tile_resolution) as f32, 0.0, 1.0);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            let start_of_vertical = n as i32;
            for i in 0..(patch_vert_resolution * 2) {
                vertices[n] = Vector3::new(0.0, 0.0, (i - tile_resolution) as f32);
                aabb.expand(vertices[n]);
                vertices[n + 1] = Vector3::new(1.0, 0.0, (i - tile_resolution) as f32);
                aabb.expand(vertices[n + 1]);
                n += 2;
            }

            let mut n = 0;
            for i in 0..(tile_resolution * 2 + 1) {
                let bl = i * 2 + 0;
                let br = i * 2 + 1;
                let tl = i * 2 + 2;
                let tr = i * 2 + 3;

                indices[n] = br;
                indices[n + 1] = bl;
                indices[n + 2] = tr;
                indices[n + 3] = bl;
                indices[n + 4] = tl;
                indices[n + 5] = tr;
                n += 6;
            }

            for i in 0..(tile_resolution * 2 + 1) {
                if i == tile_resolution { continue };

                let bl = i * 2 + 0;
                let br = i * 2 + 1;
                let tl = i * 2 + 2;
                let tr = i * 2 + 3;

                indices[n] = start_of_vertical + br;
                indices[n + 1] = start_of_vertical + tr;
                indices[n + 2] = start_of_vertical + bl;
                indices[n + 3] = start_of_vertical + bl;
                indices[n + 4] = start_of_vertical + tr;
                indices[n + 5] = start_of_vertical + tl;
                n += 6;
            }

            let cross_mesh = Self::create_mesh(&vertices, &indices, aabb);
            cross_mesh
        };

        // Seam mesh
        let seam_mesh = {
            let mut vertices = PackedVector3Array::new();
            vertices.resize((clipmap_vert_resolution * 4) as usize);
            let mut indicies = PackedInt32Array::new();
            indicies.resize((clipmap_vert_resolution * 6) as usize);

            for i in 0..clipmap_vert_resolution {
                let n = (clipmap_vert_resolution * 0 + i) as usize;
                vertices[n] = Vector3::new(i as f32, 0.0, 0.0);
                aabb.expand(vertices[n]);

                let n = (clipmap_vert_resolution * 1 + i) as usize;
                vertices[n] = Vector3::new(clipmap_vert_resolution as f32, 0.0, i as f32);
                aabb.expand(vertices[n]);
                
                let n = (clipmap_vert_resolution * 2 + i) as usize;
                vertices[n] = Vector3::new((clipmap_vert_resolution - i) as f32, 0.0, clipmap_vert_resolution as f32);
                aabb.expand(vertices[n]);

                let n = (clipmap_vert_resolution * 3 + i) as usize;
                vertices[n] = Vector3::new(0.0, 0.0, (clipmap_vert_resolution - i) as f32);
                aabb.expand(vertices[n]);
            }

            let mut n = 0;
            for i in (0..clipmap_vert_resolution * 4).step_by(2) {
                indicies[n] = i + 1;
                indicies[n + 1] = i;
                indicies[n + 2] = i + 2;
                n += 3;
            }

            let len = indicies.len();
            indicies[len - 1] = 0;
            let seam_mesh = Self::create_mesh(&vertices, &indicies, aabb);
            seam_mesh
        };

        vec![
            tile_mesh,
            filler_mesh,
            trim_mesh,
            cross_mesh,
            seam_mesh,
            tile_inner_mesh,
            filler_inner_mesh,
            trim_inner_mesh,
        ]
    }
}
