use std::fs;
use std::io;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use image::{GenericImage, RgbaImage};
use serde_json::to_writer_pretty;
use crate::constant::ImageExtension;
use crate::sprite_sheet::SpriteSheet;

pub fn collect_image_paths(dir_path: String) -> io::Result<Vec<String>> {
    let mut image_paths = vec![];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ImageExtension::from_extension(ext.to_str().unwrap_or("")).is_some() {
                    image_paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(image_paths)
}

pub fn save_texture_sheet(texture_sheet: &RgbaImage, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let output_file = output_dir.join(format!("texture_sheet_{}.png", sheet_count));
    texture_sheet.save(output_file).expect("Failed to save texture sheet");
    println!("Saved texture sheet {}", sheet_count);
    Ok(())
}

pub fn save_texture_sheet_json(sprite_sheet: &SpriteSheet, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let json_file = output_dir.join(format!("texture_sheet_{}.json", sheet_count));
    let file = File::create(json_file)?;
    to_writer_pretty(file, &sprite_sheet.to_json()).expect("Failed to write JSON");

    println!("Saved texture sheet JSON {}", sheet_count);
    Ok(())
}

pub fn save_css_animation_file(sprite_sheet: &SpriteSheet, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let output_file = output_dir.join(format!("texture_sheet_{}.png", sheet_count));
    let css_file = output_dir.join(format!("texture_sheet_{}.css", sheet_count));
    let mut file = File::create(css_file)?;
    let css_content = sprite_sheet.to_css_animation(output_file);

    writeln!(file, "{}", css_content)?;
    println!("Saved texture sheet CSS {}", sheet_count);
    Ok(())
}

pub fn generate_texture_sheets(sprite_sheet: &SpriteSheet, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let mut texture_sheet = RgbaImage::new(sprite_sheet.total_width, sprite_sheet.total_height);

    for sprite in &sprite_sheet.sprites {
        let img = &sprite.image;
        texture_sheet
            .copy_from(img, sprite.x, sprite.y)
            .expect("Failed to copy image");
    }

    save_texture_sheet(&texture_sheet, output_dir, sheet_count)?;
    save_texture_sheet_json(sprite_sheet, output_dir, sheet_count)?;
    save_css_animation_file(sprite_sheet, output_dir, sheet_count)?;

    Ok(())
}