use std::fs;
use std::io;
use std::path::Path;
use std::fs::File;
use image::{GenericImage, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use crate::constant::ImageExtension;
use crate::sprite::Sprite;
use crate::sprite_sheet::SpriteSheet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSprite {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl JsonSprite {
    pub fn new(sprite: &Sprite) -> Self {
        JsonSprite {
            name: sprite.name.clone(),
            x: sprite.x,
            y: sprite.y,
            width: sprite.width,
            height: sprite.height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSpriteSheet {
    pub sprites: Vec<JsonSprite>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}


pub fn collect_image_paths(dir_path: String) -> io::Result<Vec<String>> {
    let mut image_paths = vec![];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(image_ext) = ImageExtension::from_extension(ext.to_str().unwrap_or("")) {
                    println!("Found image with extension: {:?}", image_ext);
                    image_paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        std::process::exit(1);
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
    let mut sprites = Vec::new();
    for (_i, sprite) in sprite_sheet.sprites.iter().enumerate() {
        sprites.push(JsonSprite::new(sprite))
    }

    let texture_sheet = JsonSpriteSheet {
        sprites,
        sheet_width: sprite_sheet.total_width,
        sheet_height: sprite_sheet.total_height,
    };

    let json_file = output_dir.join(format!("texture_sheet_{}.json", sheet_count));
    let file = File::create(json_file)?;
    to_writer_pretty(file, &texture_sheet).expect("Failed to write JSON");

    println!("Saved texture sheet JSON {}", sheet_count);
    Ok(())
}

pub fn save_texture_sheet_and_json(
    texture_sheet: &RgbaImage,
    sprite_sheet: &SpriteSheet,
    output_dir: &Path,
    sheet_count: usize,
) -> io::Result<()> {
    save_texture_sheet(texture_sheet, output_dir, sheet_count)?;
    save_texture_sheet_json(sprite_sheet, output_dir, sheet_count)?;
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

    save_texture_sheet_and_json(
        &texture_sheet,
        sprite_sheet,
        output_dir,
        sheet_count,
    )?;

    Ok(())
}