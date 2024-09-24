use std::io;
use std::path::Path;
use image::{DynamicImage, GenericImageView, RgbaImage, GenericImage};
use crate::texture_sheet::Sprite;
use crate::constant::{MAX_SHEET_HEIGHT, MAX_SHEET_WIDTH};
use crate::file_io::{save_texture_sheet, save_texture_sheet_json};

pub fn generate_texture_sheets(images: &[DynamicImage], output_dir: &Path) -> io::Result<()> {
    let mut texture_sheet = RgbaImage::new(MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;

    let mut sheet_count = 1;
    let mut sprite_infos = Vec::new();

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

        x_offset += img_width;
        row_height = row_height.max(img_height);
    }

    save_texture_sheet(&texture_sheet, output_dir, sheet_count)?;
    save_texture_sheet_json(&sprite_infos, output_dir, sheet_count, MAX_SHEET_WIDTH, y_offset + row_height)?;

    Ok(())
}