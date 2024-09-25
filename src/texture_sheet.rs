use serde::{Serialize, Deserialize};
use image::{DynamicImage, GenericImageView};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteSheet {
    pub sprites: Vec<Sprite>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SheetPlacement {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub row_height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SheetLayout {
    pub placements: Vec<SheetPlacement>,
    pub total_width: u32,
    pub total_height: u32,
}

pub fn calculate_sheet_dimensions(images: &[DynamicImage], max_sheet_width: u32, max_sheet_height: u32) -> SheetLayout {
    let mut placements = Vec::new();
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;
    let mut total_height = 0;
    let mut current_max_width = 0;

    for img in images {
        let img_width = img.width();
        let img_height = img.height();

        if x_offset + img_width <= max_sheet_width {
            placements.push(SheetPlacement {
                x: x_offset,
                y: y_offset,
                width: img_width,
                height: img_height,
                row_height,
            });
            x_offset += img_width;
            row_height = row_height.max(img_height);
        } else {
            x_offset = 0;
            y_offset += row_height;
            total_height += row_height;
            row_height = img_height;
            placements.push(SheetPlacement {
                x: x_offset,
                y: y_offset,
                width: img_width,
                height: img_height,
                row_height,
            });
            x_offset += img_width;
        }
        current_max_width = current_max_width.max(x_offset);
    }

    total_height += row_height;

    let final_height = total_height.min(max_sheet_height);
    SheetLayout {
        placements,
        total_width: current_max_width,
        total_height: final_height,
    }
}


