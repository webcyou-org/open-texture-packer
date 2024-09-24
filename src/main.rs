mod texture_sheet;
mod file_io;
mod sprite_packer;
mod constant;

use std::fs;
use std::path::Path;
use std::env;
use file_io::collect_image_paths;
use sprite_packer::generate_texture_sheets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_directory> [output_directory]", args[0]);
        std::process::exit(1);
    }

    let input_dir = &args[1];

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
    generate_texture_sheets(&images, output_dir)?;

    Ok(())
}
