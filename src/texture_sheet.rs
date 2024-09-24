use serde::{Serialize, Deserialize};

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
