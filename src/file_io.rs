use std::fs;
use std::io;
use std::path::Path;
use std::fs::File;
use image::RgbaImage;
use serde_json::to_writer_pretty;
use crate::texture_sheet::{Sprite, SpriteSheet};

pub fn collect_image_paths(dir_path: &String) -> io::Result<Vec<String>> {
    let mut image_paths = vec![];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "png" || ext == "jpg" || ext == "jpeg" {
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

pub fn save_texture_sheet_json(sprites: &[Sprite], output_dir: &Path, sheet_count: usize, sheet_width: u32, sheet_height: u32) -> io::Result<()> {
    let texture_sheet = SpriteSheet {
        sprites: sprites.to_vec(),
        sheet_width,
        sheet_height,
    };

    let json_file = output_dir.join(format!("texture_sheet_{}.json", sheet_count));
    let file = File::create(json_file)?;
    to_writer_pretty(file, &texture_sheet).expect("Failed to write JSON");

    println!("Saved texture sheet JSON {}", sheet_count);
    Ok(())
}
