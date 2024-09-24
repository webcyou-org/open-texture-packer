mod texture_sheet;
mod file_io;
mod sprite_packer;
mod constant;

use std::path::Path;
use std::env;
use file_io::collect_image_paths;
use sprite_packer::generate_texture_sheets;

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
