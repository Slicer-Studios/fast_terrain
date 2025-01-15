mod fast_terrain_assets_resource;
mod fast_terrain_assets;
mod fast_terrain_mesh_asset;
mod fast_terrain_texture_asset;
mod generated_texture;
mod geoclipmap;
mod types;

use godot::{classes::RenderingServer, prelude::*};

struct FastTerrainExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FastTerrainExtension {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Editor => {
                godot_print!("FastTerrain initialized!");
            }
            _ => {}
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct FastTerrain {
    #[export]
    region_size: RegionSize,

    data_directory: GString,
    is_inside_world: bool,
    initialized: bool,
    warnings: u8,

    base: Base<Node3D>,
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
enum RegionSize {
    Size64 = 64,
    Size128 = 128,
    Size256 = 256,
    Size512 = 512,
    Size1024 = 1024,
    Size2048 = 2048,
}

#[godot_api]
impl INode3D for FastTerrain {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            region_size: RegionSize::Size256,
            data_directory: "".into(),
            is_inside_world: false,
            initialized: false,
            warnings: 0,
            base,
        }
    }

    fn ready(&mut self) {
        let new_node = RenderingServer::singleton().instance_create();
        if new_node.is_valid() {
        // self.base().get_tree().unwrap().get_root().unwrap().add_child(&new_node);
        }
        // self.base().get_tree().unwrap().get_root().unwrap().add_child(

    }
}

impl FastTerrain {
    fn build_meshes(&mut self, lods: i8, size: i32) {
        godot_print!("Building meshes with {} LODs and size {}", lods, size);
    }
}