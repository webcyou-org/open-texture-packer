use std::io;
use std::path::Path;
use image::{DynamicImage, RgbaImage, GenericImage};
use crate::texture_sheet::{Sprite, SheetLayout};
use crate::file_io::save_texture_sheet_and_json;

pub fn generate_texture_sheets(images: &[DynamicImage], layout: &SheetLayout, output_dir: &Path, sheet_count: usize) -> io::Result<()> {
    let mut texture_sheet = RgbaImage::new(layout.total_width, layout.total_height);
    let mut sprite_infos = Vec::new();

    for placement in &layout.placements {
        let img = &images[placement.image_index];

        texture_sheet.copy_from(img, placement.x, placement.y)
            .expect("Failed to copy image");

        sprite_infos.push(Sprite {
            name: format!("sprite_{}", placement.image_index),
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
        });
    }

    save_texture_sheet_and_json(&texture_sheet, &sprite_infos, output_dir, sheet_count, layout.total_width, layout.total_height)?;

    Ok(())
}


