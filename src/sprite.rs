use image::{DynamicImage, GenericImageView};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub image: DynamicImage,
    pub image_index: u32,
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
            name,
            x: 0,
            y: 0,
            width,
            height,
            image,
            image_index: 0
        })
    }

    pub fn update(&mut self, x: u32, y: u32, image_index: u32) {
        self.x = x;
        self.y = y;
        self.image_index = image_index;
    }
}