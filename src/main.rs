use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct Sprite {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpriteSheet {
    sprites: Vec<Sprite>,
    sheet_width: u32,
    sheet_height: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_paths = vec!["sprite1.png", "sprite2.png", "sprite3.png"];
    let mut images: Vec<DynamicImage> = vec![];

    for path in &image_paths {
        let img = image::open(path)?;
        images.push(img);
    }

    // テクスチャシートの幅と高さを計算 (ここでは単純に縦に並べる)
    let sheet_width = images.iter().map(|img| img.width()).max().unwrap_or(0);
    let sheet_height = images.iter().map(|img| img.height()).sum();

    // テクスチャシートを作成
    let mut texture_sheet = ImageBuffer::new(sheet_width, sheet_height);
    let mut y_offset = 0;
    let mut sprites = vec![];

    // 各画像をシートに配置
    for (i, img) in images.iter().enumerate() {
        texture_sheet.copy_from(img, 0, y_offset).expect("Failed to copy image");

        // 各スプライトの情報を保存
        sprites.push(Sprite {
            name: format!("sprite_{}", i),
            x: 0,
            y: y_offset,
            width: img.width(),
            height: img.height(),
        });

        y_offset += img.height();
    }

    // テクスチャシートを保存
    texture_sheet.save("texture_sheet.png")?;

    // スプライトの情報をJSONに保存
    let sprite_sheet = SpriteSheet {
        sprites,
        sheet_width,
        sheet_height,
    };

    let json = serde_json::to_string_pretty(&sprite_sheet)?;
    let mut file = File::create("sprite_sheet.json")?;
    file.write_all(json.as_bytes())?;

    println!("テクスチャシートとJSONファイルを作成しました。");
    Ok(())
}
