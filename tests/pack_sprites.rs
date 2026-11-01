//! End-to-end checks: from sprites to the written PNG and JSON files.

use image::{DynamicImage, Rgba, RgbaImage};
use open_texture_packer::{
    calculate_sheet_dimensions, generate_texture_sheets, pack_sprites, JsonSpriteSheet,
    PackingAlgorithm, Sprite,
};
use std::fs;
use std::path::Path;

fn color_of(index: usize) -> Rgba<u8> {
    Rgba([index as u8 + 1, 255 - index as u8, 128, 200])
}

fn sprite(index: usize, width: u32, height: u32) -> Sprite {
    let image = RgbaImage::from_pixel(width, height, color_of(index));
    Sprite {
        index: 0,
        name: format!("sprite_{index}.png"),
        x: 0,
        y: 0,
        width,
        height,
        image: DynamicImage::ImageRgba8(image),
        texture_index: 0,
    }
}

fn mixed_sprites() -> Vec<Sprite> {
    let sizes = [
        (40, 30),
        (12, 50),
        (64, 8),
        (25, 25),
        (9, 9),
        (30, 44),
        (70, 20),
        (18, 33),
        (50, 12),
        (22, 61),
        (35, 35),
        (8, 40),
        (44, 17),
        (60, 28),
        (15, 15),
        (27, 52),
        (33, 9),
        (48, 48),
    ];
    sizes
        .iter()
        .enumerate()
        .map(|(i, &(w, h))| sprite(i, w, h))
        .collect()
}

#[test]
fn written_png_matches_written_json() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("pack_sprites");

    for algorithm in PackingAlgorithm::ALL {
        let dir = root.join(format!("{algorithm:?}"));
        fs::create_dir_all(&dir).unwrap();

        let sheets = pack_sprites(mixed_sprites(), algorithm, 128, 128).unwrap();
        let mut seen = 0;

        for (i, sheet) in sheets.iter().enumerate() {
            generate_texture_sheets(sheet, &dir, i + 1).unwrap();

            let png = image::open(dir.join(format!("texture_sheet_{}.png", i + 1)))
                .unwrap()
                .to_rgba8();
            let json =
                fs::read_to_string(dir.join(format!("texture_sheet_{}.json", i + 1))).unwrap();
            let json: JsonSpriteSheet = serde_json::from_str(&json).unwrap();

            assert_eq!(png.dimensions(), (json.sheet_width, json.sheet_height));

            for entry in &json.sprites {
                let index: usize = entry.name["sprite_".len()..entry.name.len() - ".png".len()]
                    .parse()
                    .unwrap();
                for y in entry.y..entry.y + entry.height {
                    for x in entry.x..entry.x + entry.width {
                        assert_eq!(
                            *png.get_pixel(x, y),
                            color_of(index),
                            "{algorithm}: {} at ({x}, {y})",
                            entry.name
                        );
                    }
                }
                seen += 1;
            }
        }

        assert_eq!(seen, mixed_sprites().len(), "{algorithm}");
    }
}

#[test]
fn frames_stay_in_input_order_on_each_sheet() {
    for algorithm in PackingAlgorithm::ALL {
        let sheets = pack_sprites(mixed_sprites(), algorithm, 128, 128).unwrap();

        for (sheet_index, sheet) in sheets.iter().enumerate() {
            let inputs: Vec<usize> = sheet
                .sprites
                .iter()
                .map(|s| {
                    s.name["sprite_".len()..s.name.len() - ".png".len()]
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(
                inputs.windows(2).all(|pair| pair[0] < pair[1]),
                "{algorithm}"
            );

            for (frame, sprite) in sheet.sprites.iter().enumerate() {
                assert_eq!(sprite.index, frame as u32, "{algorithm}");
                assert_eq!(sprite.texture_index, sheet_index as u32, "{algorithm}");
            }
        }
    }
}

#[test]
fn next_fit_matches_the_original_packer() {
    for (max_width, max_height) in [(128, 128), (100, 200), (4096, 4096), (70, 61)] {
        let mut original_input = mixed_sprites();
        let original = calculate_sheet_dimensions(&mut original_input, max_width, max_height);
        let packed = pack_sprites(
            mixed_sprites(),
            PackingAlgorithm::NextFit,
            max_width,
            max_height,
        )
        .unwrap();

        assert_eq!(original.len(), packed.len());
        for (a, b) in original.iter().zip(&packed) {
            assert_eq!(
                (a.total_width, a.total_height),
                (b.total_width, b.total_height)
            );
            assert_eq!(a.sprites.len(), b.sprites.len());
            for (s, t) in a.sprites.iter().zip(&b.sprites) {
                assert_eq!(
                    (&s.name, s.x, s.y, s.index, s.texture_index),
                    (&t.name, t.x, t.y, t.index, t.texture_index)
                );
            }
        }
    }
}

#[test]
fn oversized_sprite_is_an_error_instead_of_a_broken_sheet() {
    let error = pack_sprites(
        vec![sprite(0, 200, 10)],
        PackingAlgorithm::NextFit,
        128,
        128,
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "sprite #0 (200x10) does not fit in the maximum sheet size (128x128)"
    );
}
