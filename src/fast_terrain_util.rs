use godot::{classes::{file_access::ModeFlags, image::{CompressMode, Format, Interpolation, UsedChannels}, resource_loader::CacheMode, Engine, FileAccess, Image, ResourceLoader}, prelude::*};

use crate::{fast_terrain_region::MapType, generated_texture::GeneratedTexture};

#[derive(GodotClass)]
#[class(base=Object, tool, init)]
pub struct FastTerrainUtil {
    #[base]
    base: Base<Object>,
}

#[godot_api]
impl FastTerrainUtil {
    // Array and Dictionary printing utilities
    #[func]
    fn print_arr(name: GString, arr: Array<Variant>, level: i32) {
        godot_print!("Array[{}]: {}", arr.len(), name);
        for i in 0..arr.len() {
            let var = arr.get(i).unwrap();
            match var.get_type() {
                VariantType::ARRAY => {
                    Self::print_arr(format!("{}{}", name, i).into(), var.to::<Array<Variant>>(), level);
                }
                VariantType::DICTIONARY => {
                    Self::print_dict(format!("{}{}", name, i).into(), var.to::<Dictionary>(), level);
                }
                VariantType::OBJECT => {
                    if let Ok(instance_id) = var.try_to::<i64>() {
                        godot_print!("{}: Object#{}", i, instance_id);
                    }
                }
                _ => {
                    godot_print!("{}: {}", i, var);
                }
            }
        }
    }

    #[func]
    fn print_dict(name: GString, dict: Dictionary, level: i32) {
        godot_print!("Dictionary: {}", name);
        let keys = dict.keys_array();
        for i in 0..keys.len() {
            let key = keys.get(i).unwrap();
            let var = dict.get(key.clone()).unwrap();
            match var.get_type() {
                VariantType::ARRAY => {
                    Self::print_arr(key.clone().to::<GString>(), var.to::<Array<Variant>>(), level);
                }
                VariantType::DICTIONARY => {
                    Self::print_dict(key.clone().to::<GString>(), var.to::<Dictionary>(), level);
                }
                VariantType::OBJECT => {
                    if let Ok(instance_id) = var.try_to::<i64>() {
                        godot_print!("\"{}\": Object#{}", key, instance_id);
                    }
                }
                _ => {
                    godot_print!("\"{}\": Value: {}", key, var);
                }
            }
        }
    }

    // Location and filename utilities
    #[func]
    fn filename_to_location(filename: GString) -> Vector2i {
        let location_string = filename
            .trim_prefix("terrain3d")
            .trim_suffix(".res")
            .to_string();
        Self::string_to_location(location_string.into())
    }

    #[func]
    fn string_to_location(string: GString) -> Vector2i {
        let x_str = string.left(3).replace("_", "");
        let y_str = string.right(3).replace("_", "");
        
        if !x_str.is_valid_int() || !y_str.is_valid_int() {
            godot_error!("Malformed string '{}'. Result: {}, {}", string, x_str, y_str);
            return Vector2i::new(i32::MAX, i32::MAX);
        }
        
        Vector2i::new(x_str.to_int() as i32, y_str.to_int() as i32)
    }

    #[func]
    fn location_to_filename(region_loc: Vector2i) -> GString {
        // Expects a v2i(-1,2) and returns terrain3d-01_02.res
        format!("terrain3d{}.res", Self::location_to_string(region_loc)).into()
    }

    #[func]
    fn location_to_string(region_loc: Vector2i) -> GString {
        // Expects a v2i(-1,2) and returns -01_02
        const POS_REGION_FORMAT: &str = "_%02d";
        const NEG_REGION_FORMAT: &str = "%03d";

        let x_str = if region_loc.x >= 0 {
            format!("_{:02}", region_loc.x)
        } else {
            format!("{:03}", region_loc.x)
        };

        let y_str = if region_loc.y >= 0 {
            format!("_{:02}", region_loc.y)
        } else {
            format!("{:03}", region_loc.y)
        };

        format!("{}{}", x_str, y_str).into()
    }

    // Image utilities
    #[func]
    fn black_to_alpha(image: Gd<Image>) -> Option<Gd<Image>> {
        let width = image.get_width();
        let height = image.get_height();
        let mut img = Image::create_empty(width, height, image.has_mipmaps(), Format::RGBAF)?;
        
        for y in 0..height {
            for x in 0..width {
                let mut pixel = image.get_pixel(x, y);
                let lumincance = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
                pixel.a = lumincance;
                img.set_pixel(x, y, pixel);
            }
        }
        
        Some(img)
    }

    #[func]
    fn dump_gentex(gen: Gd<GeneratedTexture>, name: GString, _level: i32) {
        godot_print!(
            "Generated {} RID: {}, dirty: {}, image: {:?}",
            name,
            gen.bind().get_rid(),
            gen.bind().is_dirty(),
            gen.bind().get_image()
        );
    }

    #[func]
    fn dump_maps(maps: Array<Gd<Image>>, name: GString) {
        godot_print!("Dumping {} map array. Size: {}", name, maps.len());
        for (i, img) in maps.iter_shared().enumerate() {
            godot_print!(
                "[{}]: Map size: {} format: {} {:?}",
                i,
                img.get_size(),
                img.get_format().ord(),
                img
            );
        }
    }

    #[func]
    fn get_thumbnail(image: Gd<Image>, size: Vector2i) -> Option<Gd<Image>> {
        if image.is_empty() {
            godot_error!("Provided image is empty. Nothing to process");
            return None;
        }

        let size = Vector2i::new(
            size.x.clamp(8, 16384),
            size.y.clamp(8, 16384)
        );

        godot_print!("Drawing a thumbnail sized: {}", size);
        
        // Create scaled work image
        let mut img = Image::new_gd();
        img.copy_from(&image);
        img.resize_ex(size.x, size.y).interpolation(Interpolation::LANCZOS).done();

        // Get min/max height values
        let minmax = Self::get_min_max(&img);
        let hmin = minmax.x.abs();
        let mut hmax = minmax.y.abs() + hmin;
        hmax = if hmax == 0.0 { 0.001 } else { hmax };

        // Create normalized thumbnail
        let mut thumb = Image::create_empty(size.x, size.y, false, Format::RGB8)?;
        for y in 0..thumb.get_height() {
            for x in 0..thumb.get_width() {
                let mut col = img.get_pixel(x, y);
                col.r = (col.r + hmin) / hmax;
                col.g = col.r;
                col.b = col.r;
                thumb.set_pixel(x, y, col);
            }
        }

        Some(thumb)
    }

    #[func]
    fn get_filled_image(size: Vector2i, color: Color, create_mipmaps: bool, format: Format) -> Option<Gd<Image>> {
        let format = if format.ord() < Format::MAX.ord() { format } else { Format::DXT5 };

        let (compression_format, channels, format, compress, fill_image) = if format.ord() >= Format::DXT1.ord() {
            match format {
                Format::DXT1 => (
                    CompressMode::S3TC,
                    UsedChannels::RGB,
                    Format::RGB8,
                    true,
                    true
                ),
                Format::DXT5 => (
                    CompressMode::S3TC,
                    UsedChannels::RGBA,
                    Format::RGBA8,
                    true,
                    true
                ),
                Format::BPTC_RGBA => (
                    CompressMode::BPTC,
                    UsedChannels::RGBA,
                    Format::RGBA8,
                    true,
                    true
                ),
                _ => (
                    CompressMode::MAX,
                    UsedChannels::RGBA,
                    format,
                    false,
                    false
                ),
            }
        } else {
            (
                CompressMode::MAX,
                UsedChannels::RGBA,
                format,
                false,
                true
            )
        };

        let mut img = Image::create_empty(size.x, size.y, create_mipmaps, format)?;

        if fill_image {
            let mut color = color;
            if color.a < 0.0 {
                color.a = 1.0;
                let col_a = Color::from_rgba(0.8, 0.8, 0.8, 1.0) * color;
                let col_b = Color::from_rgba(0.5, 0.5, 0.5, 1.0) * color;
                
                img.fill_rect(Rect2i::new(Vector2i::ZERO, size / 2), col_a);
                img.fill_rect(Rect2i::new(size / 2, size / 2), col_a);
                img.fill_rect(Rect2i::new(Vector2i::new(size.x, 0) / 2, size / 2), col_b);
                img.fill_rect(Rect2i::new(Vector2i::new(0, size.y) / 2, size / 2), col_b);
            } else {
                img.fill(color);
            }

            if create_mipmaps {
                img.generate_mipmaps();
            }
        }

        if compress && Engine::singleton().is_editor_hint() {
            img.compress_from_channels(compression_format, channels);
        }

        Some(img)
    }

    #[func]
    fn load_image(file_name: GString, cache_mode: CacheMode, r16_height_range: Vector2, r16_size: Vector2i) -> Option<Gd<Image>> {
        if file_name.is_empty() {
            godot_error!("No file specified. Nothing imported");
            return None;
        }

        if !FileAccess::file_exists(&file_name) {
            godot_error!("File {} does not exist. Nothing to import", file_name);
            return None;
        }

        godot_print!("Attempting to load: {}", file_name);
        let ext = file_name.get_extension().to_string().to_lowercase();
        let imgloader_extensions: Array<GString> = array!("bmp", "dds", "exr", "hdr", "jpg", "jpeg", "png", "tga", "svg", "webp");

        let img = if ext == String::from("r16") || ext == String::from("raw") {
            godot_print!("Loading file as an r16");
            let mut file = FileAccess::open(&file_name, ModeFlags::READ)?;
            let r16_size = if r16_size <= Vector2i::ZERO {
                file.seek_end();
                let fsize = file.get_position();
                let fwidth = (fsize as f32 / 2.0).sqrt() as i32;
                godot_print!(
                    "Total file size is: {} calculated width: {} dimensions: {}",
                    fsize,
                    fwidth,
                    Vector2i::new(fwidth, fwidth)
                );
                file.seek(0);
                Vector2i::new(fwidth, fwidth)
            } else {
                r16_size
            };

            let mut img = Image::create_empty(r16_size.x, r16_size.y, false, MapType::FORMATS[MapType::Height as usize])?;
            
            for y in 0..r16_size.y {
                for x in 0..r16_size.x {
                    let h = file.get_16() as f32 / 65535.0;
                    let h = h * (r16_height_range.y - r16_height_range.x) + r16_height_range.x;
                    img.set_pixel(x, y, Color::from_rgba(h, 0.0, 0.0, 1.0));
                }
            }
            Some(img)
        } else if imgloader_extensions.contains(&ext.clone().into() as &GString) {
            godot_print!("ImageFormatLoader loading recognized file type: {}", ext);
            Image::load_from_file(&file_name)
        } else {
            godot_print!("Loading file as a resource");
            ResourceLoader::singleton()
                .load_ex(&file_name)
                .cache_mode(cache_mode)
                .done()?
                .try_cast::<Image>()
                .ok()
        }?;

        if img.is_empty() {
            godot_error!("File {} is empty", file_name);
            return None;
        }

        godot_print!("Loaded Image size: {} format: {}", img.get_size(), img.get_format().ord());
        Some(img)
    }

    #[func]
    fn pack_image(src_rgb: Gd<Image>, src_a: Gd<Image>, invert_green: bool, invert_alpha: bool, alpha_channel: i32) -> Option<Gd<Image>> {
        if src_rgb.get_size() != src_a.get_size() {
            godot_error!("Provided images are not the same size. Cannot pack");
            return None;
        }

        if src_rgb.is_empty() || src_a.is_empty() {
            godot_error!("Provided images are empty. Cannot pack");
            return None;
        }

        if alpha_channel < 0 || alpha_channel > 3 {
            godot_error!("Source Channel of Height/Roughness invalid. Cannot Pack");
            return None;
        }

        let mut dst = Image::create_empty(src_rgb.get_width(), src_rgb.get_height(), false, Format::RGBA8)?;
        godot_print!("Creating image from source RGB + source channel images");

        for y in 0..src_rgb.get_height() {
            for x in 0..src_rgb.get_width() {
                let mut col = src_rgb.get_pixel(x, y);
                let alpha_pixel = src_a.get_pixel(x, y);
                // Replace array indexing with proper component access
                col.a = match alpha_channel {
                    0 => alpha_pixel.r,
                    1 => alpha_pixel.g,
                    2 => alpha_pixel.b,
                    3 => alpha_pixel.a,
                    _ => 1.0, // Default value
                };
                
                if invert_green {
                    col.g = 1.0 - col.g;
                }
                if invert_alpha {
                    col.a = 1.0 - col.a;
                }
                
                dst.set_pixel(x, y, col);
            }
        }

        Some(dst)
    }

    #[func]
    fn luminance_to_height(src_rgb: Gd<Image>) -> Option<Gd<Image>> {
        if src_rgb.is_empty() {
            godot_error!("Provided images are empty. Cannot pack");
            return None;
        }

        // Calculate contrast and offset
        let mut l_max = 0.0f32;
        let mut l_min = 1.0f32;

        for y in 0..src_rgb.get_height() {
            for x in 0..src_rgb.get_width() {
                let col = src_rgb.get_pixel(x, y);
                let l = 0.299 * col.r + 0.587 * col.g + 0.114 * col.b;
                l_max = l_max.max(l);
                l_min = l_min.min(l);
            }
        }

        let lum_contrast = 1.0 / (l_max - l_min).max(1e-6);
        let mut dst = Image::create_empty(src_rgb.get_width(), src_rgb.get_height(), false, Format::RGB8)?;

        for y in 0..src_rgb.get_height() {
            for x in 0..src_rgb.get_width() {
                let col = src_rgb.get_pixel(x, y);
                let lum = 0.299 * col.r + 0.587 * col.g + 0.114 * col.b;
                let lum = (lum * lum_contrast - l_min).clamp(0.0, 1.0);
                
                // Shape the luminance
                let shaped = 0.5 - ((1.0 - 2.0 * lum).asin() / 3.0).sin();
                let new_col = Color::from_rgba(shaped, shaped, shaped, shaped);
                dst.set_pixel(x, y, new_col);
            }
        }

        Some(dst)
    }
}

// Implementation of other utility functions
impl FastTerrainUtil {
    fn get_min_max(image: &Gd<Image>) -> Vector2 {
        if image.is_empty() {
            godot_error!("Provided image is empty. Nothing to analyze");
            return Vector2::new(f32::INFINITY, f32::INFINITY);
        }

        let mut min_max = Vector2::ZERO;
        let width = image.get_width();
        let height = image.get_height();

        for y in 0..height {
            for x in 0..width {
                let col = image.get_pixel(x, y);
                min_max.x = min_max.x.min(col.r);
                min_max.y = min_max.y.max(col.r);
            }
        }

        godot_print!("Calculating minimum and maximum values of the image: {}", min_max);
        min_max
    }

    // Add remaining utility functions...
}

// Control map handling functions
impl FastTerrainUtil {
    // Bit manipulation helpers
    fn as_float(value: u32) -> f32 {
        f32::from_bits(value)
    }

    fn as_uint(value: f32) -> u32 {
        value.to_bits()
    }

    // Base texture functions
    fn get_base(pixel: u32) -> u8 {
        ((pixel >> 27) & 0x1F) as u8
    }

    fn enc_base(base: u8) -> u32 {
        ((base & 0x1F) as u32) << 27
    }

    // Overlay functions
    fn get_overlay(pixel: u32) -> u8 {
        ((pixel >> 22) & 0x1F) as u8
    }

    fn enc_overlay(over: u8) -> u32 {
        ((over & 0x1F) as u32) << 22
    }

    // Blend functions
    fn get_blend(pixel: u32) -> u8 {
        ((pixel >> 14) & 0xFF) as u8
    }

    fn enc_blend(blend: u8) -> u32 {
        ((blend & 0xFF) as u32) << 14
    }

    // UV rotation functions
    fn get_uv_rotation(pixel: u32) -> u8 {
        ((pixel >> 10) & 0xF) as u8
    }

    fn enc_uv_rotation(rotation: u8) -> u32 {
        ((rotation & 0xF) as u32) << 10
    }

    // UV scale functions
    fn get_uv_scale(pixel: u32) -> u8 {
        ((pixel >> 7) & 0x7) as u8
    }

    fn enc_uv_scale(scale: u8) -> u32 {
        ((scale & 0x7) as u32) << 7
    }

    // Flag functions
    fn is_hole(pixel: u32) -> bool {
        ((pixel >> 2) & 0x1) == 1
    }

    fn enc_hole(hole: bool) -> u32 {
        ((hole as u32) & 0x1) << 2
    }

    fn is_nav(pixel: u32) -> bool {
        ((pixel >> 1) & 0x1) == 1
    }

    fn enc_nav(nav: bool) -> u32 {
        ((nav as u32) & 0x1) << 1
    }

    fn is_auto(pixel: u32) -> bool {
        (pixel & 0x1) == 1
    }

    fn enc_auto(auto: bool) -> u32 {
        (auto as u32) & 0x1
    }
}

// Math utilities
impl FastTerrainUtil {
    fn is_power_of_2(n: i32) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }

    fn int_divide_ceil(numer: i32, denom: i32) -> i32 {
        if (numer < 0) != (denom < 0) {
            numer / denom
        } else {
            (numer + (if denom < 0 { denom + 1 } else { denom - 1 })) / denom
        }
    }

    fn int_divide_floor(numer: i32, denom: i32) -> i32 {
        if (numer < 0) != (denom < 0) {
            (numer - (if denom < 0 { denom + 1 } else { denom - 1 })) / denom
        } else {
            numer / denom
        }
    }

    fn int_divide_round(numer: i32, denom: i32) -> i32 {
        if (numer < 0) != (denom < 0) {
            (numer - (denom / 2)) / denom
        } else {
            (numer + (denom / 2)) / denom
        }
    }

    fn bilerp(v00: f32, v01: f32, v10: f32, v11: f32, pos00: Vector2, pos11: Vector2, pos: Vector2) -> f32 {
        let x2x1 = pos11.x - pos00.x;
        let y2y1 = pos11.y - pos00.y;
        let x2x = pos11.x - pos.x;
        let y2y = pos11.y - pos.y;
        let xx1 = pos.x - pos00.x;
        let yy1 = pos.y - pos00.y;
        (v00 * x2x * y2y + v01 * x2x * yy1 + v10 * xx1 * y2y + v11 * xx1 * yy1) / (x2x1 * y2y1)
    }

    fn aabb2rect(aabb: Aabb) -> Rect2 {
        Rect2::new(
            Vector2::new(aabb.position.x, aabb.position.z),
            Vector2::new(aabb.size.x, aabb.size.z)
        )
    }
}
