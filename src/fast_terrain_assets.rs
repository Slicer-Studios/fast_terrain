use godot::prelude::*;
use std::collections::HashMap;

use crate::fast_terrain_texture_asset::FastTerrainTextureAsset;
use crate::{
    fast_terrain_texture_asset::FastTerrainTextureAsset,
    fast_terrain_mesh_asset::FastTerrainMeshAsset,
    generated_texture::GeneratedTexture,
};

pub const MAX_TEXTURES: i32 = 32;  // Updated from 16 to 32
pub const MAX_MESHES: i32 = 256;   // Updated from 64 to 256

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct FastTerrainAssets {
    #[base]
    base: Base<Resource>,

    texture_list: Vec<Gd<FastTerrainTextureAsset>>,
    mesh_list: Vec<Gd<FastTerrainMeshAsset>>,
    
    // RenderServer resources
    scenario: Rid,
    viewport: Rid,
    viewport_texture: Rid,
    camera: Rid,
    key_light: Rid,
    key_light_instance: Rid,
    fill_light: Rid,
    fill_light_instance: Rid,
    mesh_instance: Rid,

    // Generated textures
    generated_albedo_textures: Option<Gd<ImageTexture>>,
    generated_normal_textures: Option<Gd<ImageTexture>>,
    
    // Texture arrays
    texture_colors: PackedColorArray,
    texture_uv_scales: PackedFloat32Array,
    texture_detiles: PackedFloat32Array,

    // Parent terrain reference
    terrain: Option<Gd<FastTerrain>>,
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum AssetType {
    Texture = 0,
    Mesh = 1,
}

impl FastTerrainAssets {
    pub const TYPE_TEXTURE: AssetType = AssetType::Texture;
    pub const TYPE_MESH: AssetType = AssetType::Mesh;
}

#[godot_api]
impl IResource for FastTerrainAssets {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            texture_list: Vec::new(),
            mesh_list: Vec::new(),
            scenario: Rid::default(),
            viewport: Rid::default(),
            viewport_texture: Rid::default(),
            camera: Rid::default(),
            key_light: Rid::default(),
            key_light_instance: Rid::default(),
            fill_light: Rid::default(),
            fill_light_instance: Rid::default(),
            mesh_instance: Rid::default(),
            generated_albedo_textures: None,
            generated_normal_textures: None,
            texture_colors: PackedColorArray::new(),
            texture_uv_scales: PackedFloat32Array::new(),
            texture_detiles: PackedFloat32Array::new(),
            terrain: None,
        }
    }
}

#[godot_api]
impl FastTerrainAssets {
    #[func]
    pub fn initialize(&mut self, terrain: Gd<FastTerrain>) {
        self.terrain = Some(terrain);
        
        let rs = RenderingServer::singleton();
        
        // Setup preview environment
        self.scenario = rs.scenario_create();
        
        self.viewport = rs.viewport_create();
        rs.viewport_set_update_mode(self.viewport, RenderingServer::VIEWPORT_UPDATE_DISABLED);
        rs.viewport_set_scenario(self.viewport, self.scenario);
        rs.viewport_set_size(self.viewport, 128, 128);
        rs.viewport_set_transparent_background(self.viewport, true);
        rs.viewport_set_active(self.viewport, true);
        
        self.viewport_texture = rs.viewport_get_texture(self.viewport);
        
        // Setup camera
        self.camera = rs.camera_create();
        rs.viewport_attach_camera(self.viewport, self.camera);
        rs.camera_set_transform(self.camera, Transform3D::from_basis_origin(
            Basis::IDENTITY,
            Vector3::new(0.0, 0.0, 3.0)
        ));
        rs.camera_set_orthogonal(self.camera, 1.0, 0.01, 1000.0);

        // Setup lights
        self.setup_lights(&rs);
        
        self.mesh_instance = rs.instance_create();
        rs.instance_set_scenario(self.mesh_instance, self.scenario);

        // Initial updates
        self.update_texture_list();
        self.update_mesh_list();
    }

    fn setup_lights(&mut self, rs: &RenderingServer) {
        self.key_light = rs.directional_light_create();
        self.key_light_instance = rs.instance_create2(self.key_light, self.scenario);
        
        let key_transform = Transform3D::IDENTITY.looking_at(
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::UP
        );
        rs.instance_set_transform(self.key_light_instance, key_transform);

        self.fill_light = rs.directional_light_create();
        rs.light_set_color(self.fill_light, Color::from_rgb(0.3, 0.3, 0.3));
        self.fill_light_instance = rs.instance_create2(self.fill_light, self.scenario);
        
        let fill_transform = Transform3D::IDENTITY.looking_at(
            Vector3::UP,
            Vector3::FORWARD
        );
        rs.instance_set_transform(self.fill_light_instance, fill_transform);
    }

    #[func]
    pub fn update_texture_list(&mut self) {
        // Implementation for texture list update
        self.generated_albedo_textures = None;
        self.generated_normal_textures = None;
        self.update_texture_files();
        self.update_texture_settings();
        self.base.emit_signal("textures_changed".into(), &[]);
    }

    fn update_texture_files(&mut self) {
        // Implementation for updating texture files
        // This would handle texture array generation and validation
    }

    fn update_texture_settings(&mut self) {
        if !self.texture_list.is_empty() {
            self.texture_colors.clear();
            self.texture_uv_scales.clear();
            self.texture_detiles.clear();

            for texture_set in &self.texture_list {
                // Update arrays with texture settings
                // Implementation details would go here
            }
        }
        self.base.emit_signal("textures_changed".into(), &[]);
    }

    #[func]
    pub fn create_mesh_thumbnails(&mut self, id: i32, size: Vector2i) {
        // Implementation for mesh thumbnail generation
        // This would use the viewport setup to render previews
    }

    #[func]
    pub fn get_texture(&self, id: i32) -> Option<Gd<FastTerrainTextureAsset>> {
        self.texture_list.get(id as usize).cloned()
    }

    #[func]
    pub fn get_texture_list(&self) -> Array<Gd<FastTerrainTextureAsset>> {
        self.texture_list.clone().into()
    }

    #[func]
    pub fn get_texture_count(&self) -> i32 {
        self.texture_list.len() as i32
    }

    #[func]
    pub fn get_albedo_array_rid(&self) -> Rid {
        self.generated_albedo_textures.as_ref()
            .map(|tex| tex.get_rid())
            .unwrap_or_default()
    }

    #[func]
    pub fn get_normal_array_rid(&self) -> Rid {
        self.generated_normal_textures.as_ref()
            .map(|tex| tex.get_rid())
            .unwrap_or_default()
    }

    #[func]
    pub fn get_texture_colors(&self) -> PackedColorArray {
        self.texture_colors.clone()
    }

    #[func]
    pub fn get_texture_uv_scales(&self) -> PackedFloat32Array {
        self.texture_uv_scales.clone()
    }

    #[func]
    pub fn get_texture_detiles(&self) -> PackedFloat32Array {
        self.texture_detiles.clone()
    }

    #[func]
    pub fn set_mesh_list(&mut self, mesh_list: Array<Gd<FastTerrainMeshAsset>>) {
        self._set_asset_list(AssetType::Mesh, mesh_list);
    }

    #[func]
    pub fn get_mesh_list(&self) -> Array<Gd<FastTerrainMeshAsset>> {
        self.mesh_list.clone().into()
    }

    #[func]
    pub fn get_mesh_count(&self) -> i32 {
        self.mesh_list.len() as i32
    }

    fn _swap_ids(&mut self, asset_type: AssetType, src_id: i32, dst_id: i32) {
        godot_print!("Swapping asset id: {} and id: {}", src_id, dst_id);
        
        let list = match asset_type {
            AssetType::Texture => &mut self.texture_list,
            AssetType::Mesh => &mut self.mesh_list,
        };

        if src_id < 0 || src_id >= list.len() as i32 {
            godot_print!("Source id out of range: {}", src_id);
            return;
        }

        let dst_id = dst_id.clamp(0, (list.len() - 1) as i32);
        if dst_id == src_id {
            return;
        }

        list.swap(src_id as usize, dst_id as usize);
        
        match asset_type {
            AssetType::Texture => self.update_texture_list(),
            AssetType::Mesh => {
                if let Some(terrain) = &self.terrain {
                    // Implement swap_ids for instancer when available
                    // terrain.get_instancer().swap_ids(src_id, dst_id);
                }
                self.update_mesh_list();
            }
        }
    }

    #[func]
    pub fn save(&self, path: GString) -> Error {
        if path.is_empty() && self.base().get_path().is_empty() {
            godot_print!("No valid path provided");
            return Error::ERR_FILE_NOT_FOUND;
        }

        let path = if !path.is_empty() {
            path
        } else {
            self.base().get_path()
        };

        if path.get_extension() == "tres" || path.get_extension() == "res" {
            godot_print!("Attempting to save external file: {}", path);
            if let Err(err) = ResourceSaver::singleton().save(self.base(), path) {
                godot_print!("Cannot save file. Error code: {}", err);
                return err;
            }
        }

        Error::OK
    }
}

impl Drop for FastTerrainAssets {
    fn drop(&mut self) {
        let rs = RenderingServer::singleton();
        rs.free_rid(self.mesh_instance);
        rs.free_rid(self.fill_light_instance);
        rs.free_rid(self.fill_light);
        rs.free_rid(self.key_light_instance);
        rs.free_rid(self.key_light);
        rs.free_rid(self.camera);
        rs.free_rid(self.viewport);
        rs.free_rid(self.scenario);
    }
}
