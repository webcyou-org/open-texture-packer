mod texture_sheet;
mod file_io;
mod sprite_packer;
mod constant;

use std::fs;
use std::path::Path;
use file_io::collect_image_paths;
use sprite_packer::generate_texture_sheets;
use texture_sheet::calculate_sheet_dimensions;
use constant::{MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT, DEFAULT_INPUT_DIR, DEFAULT_OUTPUT_DIR};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(default_value_t = String::from(DEFAULT_INPUT_DIR))]
    input_dir: String,
    #[clap(default_value_t = String::from(DEFAULT_OUTPUT_DIR))]
    output_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let input_dir = &args.input_dir;
    let output_dir = Path::new(&args.output_dir);

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let image_paths = collect_image_paths(input_dir.to_string())?;
    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        std::process::exit(1);
    }

    let images: Vec<_> = image_paths.iter().map(|path| image::open(path).unwrap()).collect();

    let layouts = calculate_sheet_dimensions(&images, MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);
    for (i, layout) in layouts.iter().enumerate() {
        generate_texture_sheets(&images, &layout, output_dir, i + 1)?;
    }

    Ok(())
}
