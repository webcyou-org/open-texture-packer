use image::{DynamicImage, GenericImageView};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSprite {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub index: u32,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub image: DynamicImage,
    pub texture_index: u32,
}

impl Sprite {
    pub fn new(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(&Path::new(&path))?;
        let (width, height) = image.dimensions();

        let name = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Sprite {
            index: 0,
            name,
            x: 0,
            y: 0,
            width,
            height,
            image,
            texture_index: 0
        })
    }

    pub fn update(&mut self, x: u32, y: u32, index: u32, texture_index: u32) {
        self.index = index;
        self.x = x;
        self.y = y;
        self.texture_index = texture_index;
    }

    pub fn to_json(&self) -> JsonSprite {
        JsonSprite {
            name: self.name.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_frame_percentage(&self, total_frames: u32) -> f64 {
        if self.index == 0 {
            0.0
        } else if self.index == total_frames - 1 {
            100.0
        } else {
            self.index as f64 * 100.0 / (total_frames - 1) as f64
        }
    }

    pub fn get_css_animation_frame_property(&self, total_frames: u32) -> String {
        format!(
            "{:.3}% {{ background-position: -{}px -{}px; }}\n",
            self.get_frame_percentage(total_frames), self.x, self.y
        )
    }
}