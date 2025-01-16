use godot::classes::base_material_3d::{CullMode, DistanceFadeMode, Feature, Flags, Transparency};
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::classes::rendering_server::ShadowCastingSetting;
use godot::classes::{ArrayMesh, ImageTexture, Material, Mesh, MeshInstance3D, StandardMaterial3D};
use godot::meta::ParamType;
use godot::prelude::*;
use crate::fast_terrain_assets::{AssetType, MAX_MESHES};
use crate::fast_terrain_assets_resource::{FastTerrainAssetResource, FastTerrainAssetResourceImpl};

#[derive(GodotConvert, Var, Export, PartialEq, Debug)]
#[godot(via = GString)]
enum GenType {
    None,
    TextureCard,
    Max,
}

#[derive(GodotClass)]
#[class(tool, base=Resource)]
pub struct FastTerrainMeshAsset {
    #[base]
    base: Base<Resource>,

    name: GString,
    id: i32,
    height_offset: f32,
    visibility_range: f32,
    visibility_margin: f32,
    cast_shadows: ShadowCastingSetting,
    generated_faces: i32,
    generated_size: Vector2,
    density: f32,
    generated_type: GenType,
    
    packed_scene: Option<Gd<PackedScene>>,
    material_override: Option<Gd<Material>>,
    meshes: Vec<Gd<Mesh>>,
    thumbnail: Option<Gd<ImageTexture>>,
}

#[godot_api]
impl IResource for FastTerrainMeshAsset {
    fn init(base: Base<Resource>) -> Self {
        let mut instance = Self {
            base,
            name: "New Mesh".into(),
            id: 0,
            height_offset: 0.0,
            visibility_range: 100.0,
            visibility_margin: 0.0,
            cast_shadows: ShadowCastingSetting::ON,
            generated_faces: 2,
            generated_size: Vector2::new(1.0, 1.0),
            density: 10.0,
            generated_type: GenType::TextureCard,
            packed_scene: None,
            material_override: None,
            meshes: Vec::new(),
            thumbnail: None,
        };
        instance.set_generated_type(GenType::TextureCard);
        instance
    }
}

impl FastTerrainAssetResource for FastTerrainMeshAsset {
    fn clear(&mut self) {
        self.name = "New Mesh".into();
        self.id = 0;
        self.height_offset = 0.0;
        self.visibility_range = 100.0;
        self.visibility_margin = 0.0;
        self.cast_shadows = ShadowCastingSetting::ON;
        self.generated_faces = 2;
        self.generated_size = Vector2::new(1.0, 1.0);
        self.density = 10.0;
        self.packed_scene = None;
        self.material_override = None;
        self.set_generated_type(GenType::TextureCard);
        self.base_mut().notify_property_list_changed();
    }

    fn set_name(&mut self, name: GString) {
        godot_print!("Setting name: {}", name);
        self.name = name;
        self.base_mut().emit_signal("setting_changed", &[]);
    }

    fn get_name(&self) -> GString {
        self.name.clone()
    }

    fn set_id(&mut self, new_id: i32) {
        let old_id = self.id;
        let clamped_id = new_id.clamp(0, MAX_MESHES);
        godot_print!("Setting mesh id: {}", clamped_id);
        
        self.id = clamped_id;
        
        self.base_mut().emit_signal("id_changed", &[
            AssetType::Mesh.to_variant(),
            old_id.to_variant(),
            clamped_id.to_variant()
        ]);
    }

    fn get_id(&self) -> i32 {
        self.id
    }
}

impl FastTerrainAssetResourceImpl for FastTerrainMeshAsset {}

#[godot_api]
impl FastTerrainMeshAsset {
    fn set_generated_type(&mut self, gen_type: GenType) {
        godot_print!("Setting is_generated: {:?}", gen_type);
        
        if (gen_type != GenType::None) && (gen_type != GenType::Max) {
            self.packed_scene = None;
            self.meshes.clear();
            godot_print!("Generating card mesh");
            if let Some(mesh) = self.get_generated_mesh() {
                self.meshes.push(mesh);
                self.set_material_override(self.get_material());
            }
        }
        self.generated_type = gen_type;
    }

    fn get_generated_mesh(&self) -> Option<Gd<Mesh>> {
        godot_print!("Regenerating new mesh");
        let mut array_mesh = ArrayMesh::new_gd();
        let mut vertices = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut tangents = PackedFloat32Array::new();
        let mut uvs = PackedVector2Array::new();
        let mut indices = PackedInt32Array::new();

        let start_pos = Vector2::new(self.generated_size.x * -0.5, -0.5);
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let up = Vector3::UP;

        let mut point = 0;
        let mut thisrow = point;
        let mut prevrow = 0;

        for m in 1..=self.generated_faces {
            let mut z = start_pos.y;
            let angle = if m > 1 {
                (m - 1) as f32 * std::f32::consts::PI / self.generated_faces as f32
            } else {
                0.0
            };

            for j in 0..=1 {
                let mut x = start_pos.x;
                for i in 0..=1 {
                    let u = i as f32;
                    let v = j as f32;

                    let pos = Vector3::new(-x, z, 0.0).rotated(up, angle);
                    vertices.push(pos);
                    normals.push(normal);
                    
                    // ADD_TANGENT macro equivalent
                    tangents.push(1.0);
                    tangents.push(0.0);
                    tangents.push(0.0);
                    tangents.push(1.0);
                    
                    uvs.push(Vector2::new(1.0 - u, 1.0 - v));
                    
                    if i > 0 && j > 0 {
                        indices.push(prevrow + i - 1);
                        indices.push(prevrow + i);
                        indices.push(thisrow + i - 1);
                        indices.push(prevrow + i);
                        indices.push(thisrow + i);
                        indices.push(thisrow + i - 1);
                    }
                    
                    x += self.generated_size.x;
                    point += 1;
                }
                z += self.generated_size.y;
                prevrow = thisrow;
                thisrow = point;
            }
        }

        let mut arrays = Array::new();
        arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
        arrays.set(ArrayType::VERTEX.ord() as usize, vertices.to_variant().owned_to_arg());
        arrays.set(ArrayType::NORMAL.ord() as usize, normals.to_variant().owned_to_arg());
        arrays.set(ArrayType::TANGENT.ord() as usize, tangents.to_variant().owned_to_arg());
        arrays.set(ArrayType::TEX_UV.ord() as usize, uvs.to_variant().owned_to_arg());
        arrays.set(ArrayType::INDEX.ord() as usize, indices.to_variant().owned_to_arg());

        array_mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        Some(array_mesh.upcast())
    }

    fn get_material(&self) -> Option<Gd<Material>> {
        if let Some(mat) = &self.material_override {
            Some(mat.clone())
        } else {
            let mut mat = StandardMaterial3D::new_gd();
            mat.set_transparency(Transparency::ALPHA_DEPTH_PRE_PASS);
            mat.set_cull_mode(CullMode::DISABLED);
            mat.set_feature(Feature::BACKLIGHT, true);
            mat.set_backlight(Color::from_rgb(0.5, 0.5, 0.5));
            mat.set_flag(Flags::ALBEDO_FROM_VERTEX_COLOR, true);
            mat.set_distance_fade(DistanceFadeMode::PIXEL_ALPHA);
            mat.set_distance_fade_min_distance(85.0);
            mat.set_distance_fade_max_distance(75.0);
            Some(mat.upcast())
        }
    }

    // Add getters and setters
    #[func]
    pub fn set_height_offset(&mut self, offset: f32) {
        self.height_offset = offset.clamp(-50.0, 50.0);
        godot_print!("Setting height offset: {}", self.height_offset);
        self.base_mut().emit_signal("setting_changed", &[]);
    }

    #[func]
    pub fn set_scene_file(&mut self, scene_file: Option<Gd<PackedScene>>) {
        godot_print!("Setting scene file and instantiating node: {:?}", scene_file);
        self.packed_scene = scene_file.clone();

        if let Some(scene) = scene_file {
            let node = scene.instantiate();
            if node.is_none() {
                godot_print!("Error: Drag a non-empty glb, fbx, or tscn file into the scene_file slot");
                self.packed_scene = None;
                return;
            }
            let node = node.unwrap();

            if self.generated_type != GenType::None && self.generated_type != GenType::Max {
                // Reset for receiving a scene file
                self.generated_type = GenType::None;
                self.material_override = None;
                self.height_offset = 0.0;
            }

            godot_print!("Loaded scene with parent node: {:?}", node);
            let mesh_instances = node.find_children_ex("*").type_("MeshInstance3D").recursive(true).done();
            self.meshes.clear();

            for mesh_instance in mesh_instances.iter_shared() {
                if let Ok(mi) = mesh_instance.try_cast::<MeshInstance3D>() {
                    godot_print!("Found mesh: {}", mi.get_name());
                    if self.name == "New Mesh".into() {
                        let path = scene.get_path();
                        self.name = path.get_file().get_basename();
                        godot_print!("Setting name based on filename: {}", self.name);
                    }

                    if let Some(mut mesh) = mi.get_mesh() {
                        for j in 0..mi.get_surface_override_material_count() {
                            let material = if let Some(mat) = &self.material_override {
                                mat.clone()
                            } else {
                                mi.get_active_material(j).unwrap()
                            };
                            mesh.surface_set_material(j, &material);
                        }
                        self.meshes.push(mesh);
                    }
                }
            }

            if !self.meshes.is_empty() {
                let volume = self.meshes[0].get_aabb().volume();
                self.density = (10.0 / volume).clamp(0.01, 10.0);
            } else {
                godot_print!("Error: No MeshInstance3D found in scene file");
            }
            self.base_mut().notify_property_list_changed();
        } else {
            self.set_generated_type(GenType::TextureCard);
            self.density = 10.0;
        }

        godot_print!("Emitting signals");
        self.base_mut().emit_signal("file_changed", &[]);
        self.base_mut().emit_signal("instancer_setting_changed", &[]);
    }

    #[func]
    pub fn set_density(&mut self, density: f32) {
        godot_print!("Setting mesh density: {}", density);
        self.density = density.clamp(0.01, 10.0);
    }

    #[func]
    pub fn get_density(&self) -> f32 {
        self.density
    }

    #[func]
    pub fn set_visibility_range(&mut self, range: f32) {
        self.visibility_range = range.clamp(0.0, 100000.0);
        godot_print!("Setting visibility range: {}", self.visibility_range);
        self.base_mut().emit_signal("instancer_setting_changed", &[]);
    }

    #[func]
    pub fn get_visibility_range(&self) -> f32 {
        self.visibility_range
    }

    #[func]
    pub fn set_cast_shadows(&mut self, cast_shadows: ShadowCastingSetting) {
        self.cast_shadows = cast_shadows;
        godot_print!("Setting shadow casting mode: {:?}", cast_shadows);
        self.base_mut().emit_signal("instancer_setting_changed", &[]);
    }

    #[func]
    pub fn get_cast_shadows(&self) -> ShadowCastingSetting {
        self.cast_shadows
    }

    #[func]
    pub fn get_mesh(&self, index: i32) -> Option<Gd<Mesh>> {
        self.meshes.get(index as usize).cloned()
    }

    #[func]
    pub fn get_mesh_count(&self) -> i32 {
        self.meshes.len() as i32
    }

    #[func]
    pub fn get_thumbnail(&self) -> Option<Gd<ImageTexture>> {
        self.thumbnail.clone()
    }

    fn set_material_override(&mut self, material: Option<Gd<Material>>) {
        godot_print!("{}: Setting material override: {:?}", self.name, material);
        self.material_override = material;
        
        if self.material_override.is_none() && self.packed_scene.is_some() {
            godot_print!("Resetting material from scene file");
            self.set_scene_file(self.packed_scene.clone());
            return;
        }

        if let Some(material) = &self.material_override {
            // First collect the mesh we want to modify
            if let Some(mesh) = self.meshes.first().cloned() {
                let surface_count = mesh.get_surface_count();
                godot_print!("Setting material for {} surfaces", surface_count);
                
                // Create a new mesh with updated materials
                let mut updated_mesh = mesh;
                for i in 0..surface_count {
                    updated_mesh.surface_set_material(i, material);
                }
                
                // Update the first mesh in the vector
                if !self.meshes.is_empty() {
                    self.meshes[0] = updated_mesh;
                }
            }
        }
    }

    // #[validate]
    // fn validate_property(&self, property: PropertyUsageInfo) -> PropertyUsageInfo {
    //     if property.name != "generated_type" && property.name.starts_with("generated_") {
    //         if self.generated_type == GenType::None {
    //             property.usage = PropertyUsage::NO_EDITOR;
    //         } else {
    //             property.usage = PropertyUsage::DEFAULT;
    //         }
    //     }
    //     property
    // }

    #[signal]
    fn id_changed();

    #[signal]
    fn file_changed();

    #[signal]
    fn setting_changed();

    #[signal]
    fn instancer_setting_changed();
}

// need to add propeties and validate https://github.com/TokisanGames/Terrain3D/blob/main/src/terrain_3d_texture_asset.cpp