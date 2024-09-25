use std::io;
use std::path::Path;
use image::{DynamicImage, RgbaImage, GenericImage};
use crate::texture_sheet::{Sprite, SheetLayout};
use crate::file_io::save_texture_sheet_and_json;

pub fn generate_texture_sheets(images: &[DynamicImage], layout: &SheetLayout, output_dir: &Path) -> io::Result<()> {
    let mut texture_sheet = RgbaImage::new(layout.total_width, layout.total_height);
    let mut sprite_infos = Vec::new();

    for (i, placement) in layout.placements.iter().enumerate() {
        let img = &images[i];
        texture_sheet.copy_from(img, placement.x, placement.y).expect("Failed to copy image");

        sprite_infos.push(Sprite {
            name: format!("sprite_{}", i),
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
        });
    }

    let sheet_count = 1;
    save_texture_sheet_and_json(&texture_sheet, &sprite_infos, output_dir, sheet_count, layout.total_width, layout.total_height)?;

    Ok(())
}


