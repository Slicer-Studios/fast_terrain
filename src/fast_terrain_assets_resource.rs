use godot::prelude::*;

pub trait FastTerrainAssetResource {
    fn clear(&mut self);
    fn set_name(&mut self, name: GString);
    fn get_name(&self) -> GString;
    fn set_id(&mut self, id: i32);
    fn get_id(&self) -> i32;
}

// Helper methods that can be used by implementations
pub trait FastTerrainAssetResourceImpl {
    fn init_resource() -> (GString, i32) {
        ("".into(), 0)
    }
}
