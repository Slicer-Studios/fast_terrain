use godot::{classes::{image::Format, Texture2D}, prelude::*};
use crate::fast_terrain_assets::{AssetType, MAX_TEXTURES};
use crate::fast_terrain_assets_resource::{FastTerrainAssetResource, FastTerrainAssetResourceImpl};

#[derive(GodotClass)]
#[class(tool, base=Resource)]
pub struct FastTerrainTextureAsset {
    #[base]
    base: Base<Resource>,

    name: GString,
    id: i32,
    albedo_color: Color,
    albedo_texture: Option<Gd<Texture2D>>,
    normal_texture: Option<Gd<Texture2D>>,
    uv_scale: f32,
    detiling: f32,
}

#[godot_api]
impl IResource for FastTerrainTextureAsset {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            name: "New Texture".into(),
            id: 0,
            albedo_color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            albedo_texture: None,
            normal_texture: None,
            uv_scale: 0.1,
            detiling: 0.0,
        }
    }
}

impl FastTerrainAssetResource for FastTerrainTextureAsset {
    fn clear(&mut self) {
        self.name = "New Texture".into();
        self.id = 0;
        self.albedo_color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
        self.albedo_texture = None;
        self.normal_texture = None;
        self.uv_scale = 0.1;
        self.detiling = 0.0;
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
        let clamped_id = new_id.clamp(0, MAX_TEXTURES);
        godot_print!("Setting texture id: {}", clamped_id);
        
        self.id = clamped_id;
        
        self.base_mut().emit_signal("id_changed", &[
            AssetType::Texture.to_variant(),
            old_id.to_variant(),
            clamped_id.to_variant()
        ]);
    }

    fn get_id(&self) -> i32 {
        self.id
    }
}

impl FastTerrainAssetResourceImpl for FastTerrainTextureAsset {}

#[godot_api]
impl FastTerrainTextureAsset {
    // Private helper functions
    fn is_valid_format(&self, texture: Option<Gd<Texture2D>>) -> bool {
        match texture {
            None => {
                godot_print!("Provided texture is null.");
                true
            }
            Some(tex) => {
                if let Some(img) = tex.get_image() {
                    let format = img.get_format().ord();
                    if format < 0 || format >= Format::MAX.ord() {
                        godot_print!("Invalid texture format. See documentation for format specification.");
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        }
    }

    fn is_power_of_2(n: i32) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }

    #[func]
    pub fn set_albedo_texture(&mut self, texture: Option<Gd<Texture2D>>) {
        godot_print!("Setting albedo texture: {:?}", texture);
        if self.is_valid_format(texture.clone()) {
            if let Some(tex) = texture.clone() {
                let path = tex.get_path();
                let filename = path.get_file().get_basename();
                
                if self.name == "New Texture".into() {
                    self.name = filename.clone();
                    godot_print!("Naming texture based on filename: {}", self.name);
                }

                if let Some(img) = tex.get_image() {
                    if !img.has_mipmaps() {
                        godot_print!("Warning: Texture '{}' has no mipmaps. Change on the Import panel if desired.", filename);
                    }
                    if img.get_width() != img.get_height() {
                        godot_print!("Warning: Texture '{}' is not square. Mipmaps might have artifacts.", filename);
                    }
                    if !Self::is_power_of_2(img.get_width()) || !Self::is_power_of_2(img.get_height()) {
                        godot_print!("Warning: Texture '{}' size is not power of 2. This is sub-optimal.", filename);
                    }
                }
            }
            self.albedo_texture = texture;
            self.base_mut().emit_signal("file_changed", &[]);
        }
    }

    #[func]
    pub fn get_albedo_texture(&self) -> Option<Gd<Texture2D>> {
        self.albedo_texture.clone()
    }

    #[func]
    pub fn set_normal_texture(&mut self, texture: Option<Gd<Texture2D>>) {
        godot_print!("Setting normal texture: {:?}", texture);
        if self.is_valid_format(texture.clone()) {
            if let Some(tex) = texture.clone() {
                let path = tex.get_path();
                let filename = path.get_file().get_basename();

                if let Some(img) = tex.get_image() {
                    if !img.has_mipmaps() {
                        godot_print!("Warning: Texture '{}' has no mipmaps. Change on the Import panel if desired.", filename);
                    }
                    if img.get_width() != img.get_height() {
                        godot_print!("Warning: Texture '{}' is not square. Not recommended. Mipmaps might have artifacts.", filename);
                    }
                    if !Self::is_power_of_2(img.get_width()) || !Self::is_power_of_2(img.get_height()) {
                        godot_print!("Warning: Texture '{}' dimensions are not power of 2. This is sub-optimal.", filename);
                    }
                }
            }
            self.normal_texture = texture;
            self.base_mut().emit_signal("file_changed", &[]);
        }
    }

    #[func]
    pub fn get_normal_texture(&self) -> Option<Gd<Texture2D>> {
        self.normal_texture.clone()
    }

    #[func]
    pub fn set_albedo_color(&mut self, color: Color) {
        godot_print!("Setting color: {:?}", color);
        self.albedo_color = color;
        self.base_mut().emit_signal("setting_changed", &[]);
    }

    #[func]
    pub fn get_albedo_color(&self) -> Color {
        self.albedo_color
    }

    #[func]
    pub fn set_uv_scale(&mut self, scale: f32) {
        let scale = scale.clamp(0.001, 2.0);
        godot_print!("Setting uv_scale: {}", scale);
        self.uv_scale = scale;
        self.base_mut().emit_signal("setting_changed", &[]);
    }

    #[func]
    pub fn get_uv_scale(&self) -> f32 {
        self.uv_scale
    }

    #[func]
    pub fn set_detiling(&mut self, detiling: f32) {
        let detiling = detiling.clamp(0.0, 1.0);
        godot_print!("Setting detiling: {}", detiling);
        self.detiling = detiling;
        self.base_mut().emit_signal("setting_changed", &[]);
    }

    #[func]
    pub fn get_detiling(&self) -> f32 {
        self.detiling
    }

    #[signal]
    fn id_changed();

    #[signal]
    fn file_changed();

    #[signal]
    fn setting_changed();
}

// need to add properties https://github.com/TokisanGames/Terrain3D/blob/main/src/terrain_3d_mesh_asset.cpp