use image::{DynamicImage, GenericImageView, ImageBuffer};
use serde::{Serialize, Deserialize};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::env;
use image::GenericImage;

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

fn collect_image_paths(dir_path: &String) -> io::Result<Vec<String>> {
    let mut image_paths = vec![];

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "png" || ext == "jpg" || ext == "jpeg" {
                    image_paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(image_paths)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let image_paths = collect_image_paths(&args[1])?;
    if image_paths.is_empty() {
        eprintln!("No image files found in the directory.");
        std::process::exit(1);
    }

    // 読み込んだ画像を処理
    let mut images: Vec<DynamicImage> = vec![];

    // 画像の読み込み
    for path in &image_paths {
        let img = image::open(path)?;
        images.push(img);
    }

    // テクスチャシートの幅と高さを計算 (縦に並べる)
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
