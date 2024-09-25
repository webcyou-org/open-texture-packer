use serde::{Serialize, Deserialize};
use image::DynamicImage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpriteSheet {
    pub sprites: Vec<Sprite>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SheetPlacement {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub row_height: u32,
    pub image_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SheetLayout {
    pub placements: Vec<SheetPlacement>,
    pub total_width: u32,
    pub total_height: u32,
}

pub fn calculate_sheet_dimensions(
    images: &[DynamicImage],
    max_sheet_width: u32,
    max_sheet_height: u32,
) -> Vec<SheetLayout> {
    let mut layouts = Vec::new();
    let mut placements = Vec::new();
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;
    let mut total_height = 0;
    let mut current_max_width = 0;

    for (index, img) in images.iter().enumerate() {
        let img_width = img.width();
        let img_height = img.height();

        if x_offset + img_width > max_sheet_width {
            x_offset = 0;
            y_offset += row_height;
            total_height += row_height;
            row_height = 0;
        }

        if y_offset + img_height > max_sheet_height {
            layouts.push(SheetLayout {
                placements: placements.clone(),
                total_width: current_max_width,
                total_height: total_height + row_height,
            });
            placements.clear();
            x_offset = 0;
            y_offset = 0;
            total_height = 0;
            row_height = 0;
            current_max_width = 0;
        }

        placements.push(SheetPlacement {
            x: x_offset,
            y: y_offset,
            width: img_width,
            height: img_height,
            row_height,
            image_index: index,
        });

        x_offset += img_width;
        row_height = row_height.max(img_height);
        current_max_width = current_max_width.max(x_offset);
    }

    total_height += row_height;
    layouts.push(SheetLayout {
        placements,
        total_width: current_max_width,
        total_height: total_height,
    });

    layouts
}
