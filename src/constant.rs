pub const MAX_SHEET_WIDTH: u32 = 4096;
pub const MAX_SHEET_HEIGHT: u32 = 4096;

// pub const MAX_SHEET_WIDTH: u32 = 500;
// pub const MAX_SHEET_HEIGHT: u32 = 300;

pub const DEFAULT_INPUT_DIR: &str = "./";
pub const DEFAULT_OUTPUT_DIR: &str = "./output";

#[derive(Debug)]
pub enum ImageExtension {
    Png,
    Jpg,
    Jpeg,
}

impl ImageExtension {
    pub fn from_extension(ext: &str) -> Option<ImageExtension> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageExtension::Png),
            "jpg" => Some(ImageExtension::Jpg),
            "jpeg" => Some(ImageExtension::Jpeg),
            _ => None,
        }
    }
}
