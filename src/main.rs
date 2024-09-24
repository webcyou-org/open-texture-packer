use image::{DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use serde::{Serialize, Deserialize};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::env;
use image::GenericImage;
use serde_json::to_writer_pretty;

const MAX_SHEET_WIDTH: u32 = 4096;
const MAX_SHEET_HEIGHT: u32 = 4096;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Sprite {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpriteSheet {
    sprites: Vec<Sprite>,
    sheet_width: u32,
    sheet_height: u32,
}

fn generate_texture_sheets(images: &[DynamicImage], output_dir: &Path) -> io::Result<()> {
    let mut texture_sheet = RgbaImage::new(MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;

    let mut sheet_count = 1;
    let mut sprite_infos = Vec::new();

    let mut image_index = 0;

    for (i, img) in images.iter().enumerate() {
        let img_width = img.width();
        let img_height = img.height();

        if x_offset + img_width > MAX_SHEET_WIDTH {
            x_offset = 0;
            y_offset += row_height;
            row_height = 0;
        }

        if y_offset + img_height > MAX_SHEET_HEIGHT {
            save_texture_sheet(&texture_sheet, output_dir, sheet_count)?;
            save_texture_sheet_json(&sprite_infos, output_dir, sheet_count, MAX_SHEET_WIDTH, y_offset + row_height)?;

            sheet_count += 1;
            sprite_infos.clear();

            texture_sheet = RgbaImage::new(MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);
            x_offset = 0;
            y_offset = 0;
            row_height = 0;
        }

        texture_sheet.copy_from(img, x_offset, y_offset).expect("Failed to copy image");

        sprite_infos.push(Sprite {
            name: format!("sprite_{}", i),
            x: x_offset,
            y: y_offset,
            width: img_width,
            height: img_height,
        });

        // println!("Placing image {} at x: {}, y: {}", i, x_offset, y_offset);
        x_offset += img_width;
        row_height = row_height.max(img_height);
        // image_index = i;
    }

    save_texture_sheet(&texture_sheet, output_dir, sheet_count)?;
    save_texture_sheet_json(&sprite_infos, output_dir, sheet_count, MAX_SHEET_WIDTH, y_offset + row_height)?;

    Ok(())
}

fn save_texture_sheet(texture_sheet: &RgbaImage, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let output_file = output_dir.join(format!("texture_sheet_{}.png", sheet_count));
    texture_sheet.save(output_file).expect("Failed to save texture sheet");
    println!("Saved texture sheet {}", sheet_count);
    Ok(())
}

fn save_texture_sheet_json(sprites: &[Sprite], output_dir: &Path, sheet_count: usize, sheet_width: u32, sheet_height: u32) -> io::Result<()> {
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

fn collect_image_paths(dir_path: &String) -> io::Result<Vec<String>> {
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

    Ok(image_paths)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new("./output");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let image_paths = collect_image_paths(&args[1])?;
    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        std::process::exit(1);
    }

    let images: Vec<_> = image_paths.iter().map(|path| image::open(path).unwrap()).collect();
    generate_texture_sheets(&images, output_dir)?;

    Ok(())
}
