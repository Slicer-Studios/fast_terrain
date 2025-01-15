use godot::{
    classes::{rendering_server::TextureLayeredType, Image, RenderingServer},
    prelude::*,
};

pub struct GeneratedTexture {
    rid: Rid,
    image: Option<Gd<Image>>,
    dirty: bool,
}

impl GeneratedTexture {
    pub fn clear(&mut self) {
        if self.rid.is_valid() {
            godot_print!("GeneratedTexture freeing {}", self.rid);
            RenderingServer::singleton().free_rid(self.rid);
        }
        
        if let Some(image) = self.image.take() {
            godot_print!("GeneratedTexture unref image {:?}", image);
            // Image is automatically dropped here
        }
        
        self.rid = Rid::new(0);
        self.dirty = true;
    }

    pub fn create_from_layers(&mut self, layers: Array<Gd<Image>>) -> Rid {
        if !layers.is_empty() {
            godot_print!("RenderingServer creating Texture2DArray, layers size: {}", layers.len());
            
            for (i, img) in layers.iter_shared().enumerate() {
                godot_print!(
                    "{}: {:?}, empty: {}, size: {:?}, format: {:?}",
                    i,
                    img,
                    img.is_empty(),
                    img.get_size(),
                    img.get_format()
                );
            }

            self.rid = RenderingServer::singleton().texture_2d_layered_create(
                &layers,
                TextureLayeredType::LAYERED_2D_ARRAY,
            );
            self.dirty = false;
        } else {
            self.clear();
        }
        self.rid
    }

    pub fn update(&self, image: Gd<Image>, layer: i32) {
        godot_print!("RenderingServer updating Texture2DArray at index: {}", layer);
        RenderingServer::singleton().texture_2d_update(self.rid, &image, layer);
    }

    pub fn create(&mut self, image: Gd<Image>) -> Rid {
        godot_print!("RenderingServer creating Texture2D");
        self.image = Some(image.clone());
        self.rid = RenderingServer::singleton().texture_2d_create(&image);
        self.dirty = false;
        self.rid
    }
}
