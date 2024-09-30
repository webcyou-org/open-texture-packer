mod file_io;
mod constant;
mod sprite;
mod sprite_sheet;

use std::fs;
use std::path::Path;
use file_io::{collect_image_paths, generate_texture_sheets};
use constant::{MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT, DEFAULT_INPUT_DIR, DEFAULT_OUTPUT_DIR, DEFAULT_FPS};
use clap::Parser;
use sprite::Sprite;
use sprite_sheet::calculate_sheet_dimensions;

#[derive(Parser)]
struct Cli {
    #[clap(default_value_t = String::from(DEFAULT_INPUT_DIR))]
    input_dir: String,
    #[clap(default_value_t = String::from(DEFAULT_OUTPUT_DIR))]
    output_dir: String,
    #[clap(short, long, default_value_t = MAX_SHEET_WIDTH)]
    width: u32,
    #[clap(short = 't', long, default_value_t = MAX_SHEET_HEIGHT)]
    height: u32,
    #[clap(short, long, default_value_t = DEFAULT_FPS)]
    fps: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let input_dir = &args.input_dir;
    let output_dir = Path::new(&args.output_dir);
    let fps = args.fps;

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let mut image_paths = collect_image_paths(input_dir.to_string())?;
    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        return Ok(())
    }
    image_paths.sort();

    let mut sprites: Vec<_> = image_paths.iter().map(|path| {
        Sprite::new(path.to_string()).unwrap()
    }).collect();

    let mut sprite_sheets = calculate_sheet_dimensions(&mut sprites, args.width, args.height);
    for (i, sprite_sheet) in sprite_sheets.iter_mut().enumerate() {
        sprite_sheet.change_fps(fps);
        generate_texture_sheets(sprite_sheet, output_dir, i + 1)?;
    }

    Ok(())
}
