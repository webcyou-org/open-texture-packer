use std::io;
use std::path::Path;
use image::{DynamicImage, GenericImageView, RgbaImage, GenericImage};
use crate::texture_sheet::Sprite;
use crate::constant::{MAX_SHEET_HEIGHT, MAX_SHEET_WIDTH};
use crate::file_io::save_texture_sheet_and_json;

fn calculate_sheet_dimensions(images: &[DynamicImage], max_width: u32, max_height: u32) -> (u32, u32) {
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;
    let mut sheet_width = 0;
    let mut sheet_height = 0;

    for img in images {
        let img_width = img.width();
        let img_height = img.height();

        if x_offset + img_width > max_width {
            x_offset = 0;
            y_offset += row_height;
            row_height = 0;
        }

        sheet_width = sheet_width.max(x_offset + img_width);
        sheet_height = sheet_height.max(y_offset + img_height);

        x_offset += img_width;
        row_height = row_height.max(img_height);

        if sheet_height > max_height {
            break;
        }
    }
    (sheet_width.min(max_width), sheet_height.min(max_height))
}

pub fn generate_texture_sheets(images: &[DynamicImage], output_dir: &Path) -> io::Result<()> {
    let (max_sheet_width, max_sheet_height) = calculate_sheet_dimensions(images, MAX_SHEET_WIDTH, MAX_SHEET_HEIGHT);

    let mut texture_sheet = RgbaImage::new(max_sheet_width, max_sheet_height);
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;

    let mut sheet_count = 1;
    let mut sprite_infos = Vec::new();

    for (i, img) in images.iter().enumerate() {
        let img_width = img.width();
        let img_height = img.height();

        if x_offset + img_width > max_sheet_width {
            x_offset = 0;
            y_offset += row_height;
            row_height = 0;
        }

        if y_offset + img_height > max_sheet_height {
            save_texture_sheet_and_json(&texture_sheet, &sprite_infos, output_dir, sheet_count, max_sheet_width, y_offset + row_height)?;

            sheet_count += 1;
            sprite_infos.clear();

            texture_sheet = RgbaImage::new(max_sheet_width, max_sheet_height);
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

    save_texture_sheet_and_json(&texture_sheet, &sprite_infos, output_dir, sheet_count, max_sheet_width, y_offset + row_height)?;

    Ok(())
}
