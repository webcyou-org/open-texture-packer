mod texture_sheet;
mod file_io;
mod sprite_packer;
mod constant;

use std::fs;
use std::path::Path;
use std::env;
use file_io::collect_image_paths;
use sprite_packer::generate_texture_sheets;
use texture_sheet::calculate_sheet_dimensions;
use constant::{MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let input_dir = if args.len() < 2 {
        String::from("./")
    } else {
        args[1].clone()
    };

    let output_dir = if args.len() >= 3 {
        Path::new(&args[2])
    } else {
        Path::new("./output")
    };

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let image_paths = collect_image_paths(input_dir)?;
    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        std::process::exit(1);
    }

    let images: Vec<_> = image_paths.iter().map(|path| image::open(path).unwrap()).collect();
    let layout = calculate_sheet_dimensions(&images, MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);
    generate_texture_sheets(&images, &layout, output_dir)?;

    Ok(())
}
