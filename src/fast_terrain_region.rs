use godot::{classes::{image::Format, resource_saver::SaverFlags, Image, ResourceSaver}, global::Error, meta::ParamType, prelude::*};
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct FastTerrainRegion {
    #[base]
    base: Base<Resource>,
    
    version: f32,
    region_size: i32,
    vertex_spacing: f32,
    height_range: Vector2,
    location: Vector2i,
    
    height_map: Option<Gd<Image>>,
    control_map: Option<Gd<Image>>,
    color_map: Option<Gd<Image>>,
    instances: Dictionary,
    
    deleted: bool,
    edited: bool,
    modified: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    Height,
    Control,
    Color,
    Max,
}

impl FastTerrainRegion {
    const FORMATS: [Format; 3] = [
        Format::RF,  // Height
        Format::RGBA8,  // Control
        Format::RGBA8,  // Color
    ];

    const TYPE_STRS: [&'static str; 3] = [
        "Height",
        "Control",
        "Color"
    ];

    const COLORS: [Color; 3] = [
        Color::from_rgb(0.0, 0.0, 0.0),     // Height
        Color::from_rgba(0.0, 0.0, 0.0, 0.0), // Control
        Color::from_rgb(1.0, 1.0, 1.0)      // Color
    ];

    fn set_version(&mut self, version: f32) {
        godot_print!("{:.3}", version);
        self.version = version;
        if self.version < FastTerrainData::CURRENT_VERSION {
            godot_warn!("Region {} version {:.3} will be updated to {:.3} upon save", 
                self.get_path(), self.version, FastTerrainData::CURRENT_VERSION);
        }
    }

    fn set_map(&mut self, map_type: MapType, image: Option<Gd<Image>>) {
        match map_type {
            MapType::Height => self.set_height_map(image),
            MapType::Control => self.set_control_map(image),
            MapType::Color => self.set_color_map(image),
            _ => godot_error!("Requested map type is invalid"),
        }
    }

    fn get_map(&self, map_type: MapType) -> Option<Gd<Image>> {
        match map_type {
            MapType::Height => self.get_height_map(),
            MapType::Control => self.get_control_map(),
            MapType::Color => self.get_color_map(),
            _ => {
                godot_error!("Requested map type is invalid");
                None
            }
        }
    }

    fn set_maps(&mut self, maps: Array<Gd<Image>>) {
        if maps.len() != MapType::Max as usize {
            godot_error!("Expected {} maps. Received {}", MapType::Max as usize - 1, maps.len());
            return;
        }
        self.region_size = 0;
        self.set_height_map(maps.get(MapType::Height as i32));
        self.set_control_map(maps.get(MapType::Control as i32));
        self.set_color_map(maps.get(MapType::Color as i32));
    }

    fn get_maps(&self) -> Array<Gd<Image>> {
        godot_print!("Retrieving maps from region: {}", self.location);
        let mut maps = Array::new();
        if let Some(map) = &self.height_map {
            maps.push(map.clone());
        }
        if let Some(map) = &self.control_map {
            maps.push(map.clone());
        }
        if let Some(map) = &self.color_map {
            maps.push(map.clone());
        }
        maps
    }

    fn set_height_map(&mut self, map: Option<Gd<Image>>) {
        godot_print!("Setting height map for region: {}", 
            if self.location.x != i32::MAX { self.location.to_string() } else { "(new)".into() });
        
        if self.region_size == 0 {
            self.set_region_size(map.as_ref().map_or(0, |m| m.get_width()));
        }
        self.height_map = self.sanitize_map(MapType::Height, map);
        self.calc_height_range();
    }

    fn set_control_map(&mut self, map: Option<Gd<Image>>) {
        godot_print!("Setting control map for region: {}", 
            if self.location.x != i32::MAX { self.location.to_string() } else { "(new)".into() });
        
        if self.region_size == 0 {
            self.set_region_size(map.as_ref().map_or(0, |m| m.get_width()));
        }
        self.control_map = self.sanitize_map(MapType::Control, map);
    }

    fn set_color_map(&mut self, map: Option<Gd<Image>>) {
        godot_print!("Setting color map for region: {}", 
            if self.location.x != i32::MAX { self.location.to_string() } else { "(new)".into() });
        
        if self.region_size == 0 {
            self.set_region_size(map.as_ref().map_or(0, |m| m.get_width()));
        }
        self.color_map = self.sanitize_map(MapType::Color, map);
        if let Some(color_map) = &self.color_map {
            if !color_map.has_mipmaps() {
                godot_print!("Color map does not have mipmaps. Generating");
                color_map.generate_mipmaps();
            }
        }
    }

    fn sanitize_maps(&mut self) {
        if self.region_size == 0 {
            godot_error!("Set region_size first");
            return;
        }
        self.height_map = self.sanitize_map(MapType::Height, self.height_map.clone());
        self.control_map = self.sanitize_map(MapType::Control, self.control_map.clone());
        self.color_map = self.sanitize_map(MapType::Color, self.color_map.clone());
    }

    fn sanitize_map(&self, map_type: MapType, map: Option<Gd<Image>>) -> Option<Gd<Image>> {
        let type_str = Self::TYPE_STRS[map_type as usize];
        let format = Self::FORMATS[map_type as usize];
        let color = Self::COLORS[map_type as usize];
        let mut result = None;

        if let Some(input_map) = map {
            if self.validate_map_size(&input_map) {
                if input_map.get_format() == format {
                    godot_print!("Map type {} correct format, size. Mipmaps: {}", 
                        type_str, input_map.has_mipmaps());
                    result = Some(input_map);
                } else {
                    godot_print!("Provided {} map wrong format: {:?}. Converting copy to: {:?}", 
                        type_str, input_map.get_format(), format);
                    
                    let mut new_map = Image::new_gd();
                    new_map.copy_from(&input_map);
                    new_map.convert(format);

                    if new_map.get_format() == format {
                        result = Some(new_map);
                    } else {
                        godot_print!("Cannot convert image to format: {:?}. Creating blank", format);
                    }
                }
            } else {
                godot_print!("Provided {} map wrong size: {}. Creating blank", 
                    type_str, input_map.get_size());
            }
        } else {
            godot_print!("No provided {} map. Creating blank", type_str);
        }

        result.or_else(|| {
            godot_print!("Making new image of type: {} and generating mipmaps: {}", 
                type_str, map_type == MapType::Color);
            
            let mut new_map = Image::new_gd();
            new_map.create(self.region_size, self.region_size, map_type == MapType::Color, format);
            new_map.fill(color);
            Some(new_map)
        })
    }

    fn validate_map_size(&self, map: &Gd<Image>) -> bool {
        let size = map.get_size();
        if size.x != size.y {
            godot_error!("Image width doesn't match height: {}", size);
            return false;
        }
        if !Self::is_power_of_2(size.x) || !Self::is_power_of_2(size.y) {
            godot_error!("Image dimensions are not a power of 2: {}", size);
            return false;
        }
        if size.x < 64 || size.y > 2048 {
            godot_error!("Image size out of bounds (64-2048): {}", size);
            return false;
        }
        if self.region_size == 0 {
            godot_error!("Region size is 0, set it or set a map first");
            return false;
        }
        if self.region_size != size.x || self.region_size != size.y {
            godot_error!("Image size doesn't match existing images in this region: {}", size);
            return false;
        }
        true
    }

    fn is_power_of_2(n: i32) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }

    fn set_height_range(&mut self, range: Vector2) {
        godot_print!("{}", range);
        if self.height_range != range {
            if self.height_range != Vector2::ZERO {
                self.modified = true;
            }
            self.height_range = range;
        }
    }

    fn calc_height_range(&mut self) {
        if let Some(height_map) = &self.height_map {
            let range = self.get_min_max(height_map);
            if self.height_range != range {
                self.height_range = range;
                self.modified = true;
                godot_print!("Recalculated new height range: {} for region: {}. Marking modified", 
                    range, if self.location.x != i32::MAX { self.location.to_string() } else { "(new)".into() });
            }
        }
    }

    fn get_min_max(&self, image: &Gd<Image>) -> Vector2 {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        
        let size = image.get_size();
        for y in 0..size.y {
            for x in 0..size.x {
                let pixel = image.get_pixel(x, y);
                let height = pixel.r;
                min = min.min(height);
                max = max.max(height);
            }
        }
        
        Vector2::new(min, max)
    }

    fn set_region_size(&mut self, size: i32) {
        if size != self.region_size {
            godot_print!("Setting region size: {}", size);
            self.region_size = size;
            self.sanitize_maps();
        }
    }

    fn get_region_size(&self) -> i32 {
        self.region_size
    }

    fn set_vertex_spacing(&mut self, spacing: f32) {
        self.vertex_spacing = spacing;
    }

    fn get_vertex_spacing(&self) -> f32 {
        self.vertex_spacing
    }

    fn set_location(&mut self, location: Vector2i) {
        godot_print!("Set location: {}", location);
        self.location = location;
    }

    fn get_location(&self) -> Vector2i {
        self.location
    }

    fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn set_edited(&mut self, edited: bool) {
        self.edited = edited;
    }

    fn is_edited(&self) -> bool {
        self.edited
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    fn is_deleted(&self) -> bool {
        self.deleted
    }

    fn get_height_map(&self) -> Option<Gd<Image>> {
        self.height_map.clone()
    }

    fn get_control_map(&self) -> Option<Gd<Image>> {
        self.control_map.clone()
    }

    fn get_color_map(&self) -> Option<Gd<Image>> {
        self.color_map.clone()
    }

    fn save(&mut self, path: GString, sixteen_bit: bool) -> Error {
        // Check if region is properly set up
        if self.location.x == i32::MAX {
            godot_error!("Region has not been setup. Location is INT32_MAX. Skipping {}", path);
            return Error::FAILED;
        }
        
        // Skip if not modified
        if !self.modified {
            godot_print!("Region {} not modified. Skipping {}", self.location, path);
            return Error::ERR_SKIP;
        }

        // Validate path
        if path.is_empty() && self.base().get_path().is_empty() {
            godot_error!("No valid path provided");
            return Error::ERR_FILE_NOT_FOUND;
        }

        // Set path if provided
        if !path.is_empty() {
            godot_print!("Setting file path for region {} to {}", self.location, path);
            self.base_mut().take_over_path(&path);
        }

        godot_print!("Writing{} region {} to {}", 
            if sixteen_bit { " 16-bit" } else { "" },
            self.location,
            self.base().get_path());

        self.set_version(FastTerrainData::CURRENT_VERSION);

        let result = if sixteen_bit {
            // Handle 16-bit saving
            if let Some(height_map) = &self.height_map {
                // Create new image for backup
                let mut original_map = Image::new_gd();
                let original_format = height_map.get_format();
                
                // Copy data from height_map
                original_map.copy_from(height_map);
                
                // Create new image for 16-bit version
                let mut rh_map = Image::new_gd();
                rh_map.copy_from(height_map);
                rh_map.convert(Format::RH);
                
                // Replace height map temporarily
                self.height_map = Some(rh_map);
                
                // Save with compression
                let save_result = ResourceSaver::singleton().save_ex(self.base()).path(&self.base().get_path()).flags(SaverFlags::COMPRESS).done();

                // Restore original height map
                let mut restored_map = Image::new_gd();
                restored_map.copy_from(&original_map);
                restored_map.convert(original_format);
                self.height_map = Some(restored_map);
                
                save_result
            } else {
                Error::ERR_INVALID_DATA
            }
        } else {
            // Regular save with compression
            ResourceSaver::singleton().save_ex(self.base()).path(&self.base().get_path()).flags(SaverFlags::COMPRESS).done()
        };

        match result {
            Error::OK => {
                self.modified = false;
                godot_print!("File saved successfully");
            }
            err => {
                godot_error!("Cannot save region file: {}. Error code: {:?}. Look up @GlobalScope Error enum in the Godot docs", 
                    self.base().get_path(), err);
            }
        }

        result
    }
}

#[godot_api]
impl IResource for FastTerrainRegion {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            version: 0.0,
            region_size: 0,
            vertex_spacing: 0.0,
            height_range: Vector2::ZERO,
            location: Vector2i::new(i32::MAX, i32::MAX),
            height_map: None,
            control_map: None,
            color_map: None,
            instances: Dictionary::new(),
            deleted: false,
            edited: false,
            modified: false,
        }
    }
}

#[godot_api]
impl FastTerrainRegion {
    #[signal]
    fn modified_changed();

    #[func]
    fn duplicate(&self, deep: bool) -> Gd<FastTerrainRegion> {
        let mut new_region = FastTerrainRegion::new_gd();
        
        if !deep {
            new_region.bind_mut().set_data(self.get_data());
        } else {
            let mut dict = Dictionary::new();
            dict.insert("version", self.version);
            dict.insert("region_size", self.region_size);
            dict.insert("vertex_spacing", self.vertex_spacing);
            dict.insert("height_range", self.height_range);
            dict.insert("modified", self.modified);
            dict.insert("deleted", self.deleted);
            dict.insert("location", self.location);
            
            if let Some(height_map) = &self.height_map {
                dict.insert("height_map", height_map.duplicate());
            }
            if let Some(control_map) = &self.control_map {
                dict.insert("control_map", control_map.duplicate());
            }
            if let Some(color_map) = &self.color_map {
                dict.insert("color_map", color_map.duplicate());
            }
            dict.insert("instances", self.instances.duplicate_deep());
            
            new_region.bind_mut().set_data(dict);
        }
        new_region
    }

    #[func]
    fn get_data(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.insert("location", self.location);
        dict.insert("deleted", self.deleted);
        dict.insert("edited", self.edited);
        dict.insert("modified", self.modified);
        dict.insert("version", self.version);
        dict.insert("region_size", self.region_size);
        dict.insert("vertex_spacing", self.vertex_spacing);
        dict.insert("height_range", self.height_range);
        dict.insert("height_map", self.height_map.clone());
        dict.insert("control_map", self.control_map.clone());
        dict.insert("color_map", self.color_map.clone());
        dict.insert("instances", self.instances.clone());
        dict
    }

    #[func]
    fn set_data(&mut self, data: Dictionary) {
        if data.contains_key("location") { self.location = data.get("location").unwrap().to::<Vector2i>(); }
        if data.contains_key("deleted") { self.deleted = data.get("deleted").unwrap().to::<bool>(); }
        if data.contains_key("edited") { self.edited = data.get("edited").unwrap().to::<bool>(); }
        if data.contains_key("modified") { self.modified = data.get("modified").unwrap().to::<bool>(); }
        if data.contains_key("version") { self.version = data.get("version").unwrap().to::<f32>(); }
        if data.contains_key("region_size") { self.region_size = data.get("region_size").unwrap().to::<i32>(); }
        if data.contains_key("vertex_spacing") { self.vertex_spacing = data.get("vertex_spacing").unwrap().to::<f32>(); }
        if data.contains_key("height_range") { self.height_range = data.get("height_range").unwrap().to::<Vector2>(); }
        if data.contains_key("height_map") { self.height_map = data.get("height_map").unwrap().to::<Option<Gd<Image>>>(); }
        if data.contains_key("control_map") { self.control_map = data.get("control_map").unwrap().to::<Option<Gd<Image>>>(); }
        if data.contains_key("color_map") { self.color_map = data.get("color_map").unwrap().to::<Option<Gd<Image>>>(); }
        if data.contains_key("instances") { self.instances = data.get("instances").unwrap().to::<Dictionary>(); }
    }
}
