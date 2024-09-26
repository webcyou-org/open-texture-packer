use serde::{Deserialize, Serialize};
use crate::sprite::{Sprite, JsonSprite};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSpriteSheet {
    pub sprites: Vec<JsonSprite>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}

#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub sprites: Vec<Sprite>,
    pub total_width: u32,
    pub total_height: u32,
}

impl SpriteSheet {
    pub fn to_json(&self) -> JsonSpriteSheet {
        let mut sprites = Vec::new();
        for (_i, sprite) in self.sprites.iter().enumerate() {
            sprites.push(sprite.to_json())
        }
        JsonSpriteSheet {
            sprites,
            sheet_width: self.total_width,
            sheet_height: self.total_height,
        }
    }
}

pub fn calculate_sheet_dimensions(
    sprites: &mut [Sprite],
    max_sheet_width: u32,
    max_sheet_height: u32,
) -> Vec<SpriteSheet> {
    let mut sheets = Vec::new();
    let mut current_sprites = Vec::new();
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut row_height = 0;
    let mut total_height = 0;
    let mut current_max_width = 0;
    let mut sheet_count = 0;

    for sprite in sprites.iter_mut() {
        let img_width = sprite.width;
        let img_height = sprite.height;

        if x_offset + img_width > max_sheet_width {
            x_offset = 0;
            y_offset += row_height;
            total_height += row_height;
            row_height = 0;
        }

        if y_offset + img_height > max_sheet_height {
            sheets.push(SpriteSheet {
                sprites: current_sprites.clone(),
                total_width: current_max_width,
                total_height: total_height + row_height,
            });
            current_sprites.clear();
            x_offset = 0;
            y_offset = 0;
            total_height = 0;
            row_height = 0;
            current_max_width = 0;
            sheet_count += 1;
        }

        sprite.update(x_offset, y_offset, sheet_count);

        current_sprites.push(sprite.clone());

        x_offset += img_width;
        row_height = row_height.max(img_height);
        current_max_width = current_max_width.max(x_offset);
    }

    total_height += row_height;
    sheets.push(SpriteSheet {
        sprites: current_sprites,
        total_width: current_max_width,
        total_height: total_height,
    });

    sheets
}
