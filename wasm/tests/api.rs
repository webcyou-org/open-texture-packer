//! Run with `wasm-pack test --node wasm`.

use open_texture_packer::{PackingAlgorithm, PackingResult};
use open_texture_packer_wasm::{algorithms, compare_algorithms_js, pack_rects_js};
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

#[derive(Serialize)]
struct Size {
    width: u32,
    height: u32,
}

fn sizes(list: &[(u32, u32)]) -> JsValue {
    let list: Vec<Size> = list
        .iter()
        .map(|&(width, height)| Size { width, height })
        .collect();
    serde_wasm_bindgen::to_value(&list).unwrap()
}

fn field(value: &JsValue, name: &str) -> JsValue {
    js_sys::Reflect::get(value, &name.into()).unwrap()
}

#[wasm_bindgen_test]
fn lists_every_algorithm_with_its_cli_id() {
    let list = js_sys::Array::from(&algorithms().ok().unwrap());
    assert_eq!(list.length() as usize, PackingAlgorithm::ALL.len());

    let first = list.get(0);
    assert_eq!(field(&first, "id").as_string().unwrap(), "shelf-packing");
    assert_eq!(field(&first, "name").as_string().unwrap(), "Shelf Packing");
}

#[wasm_bindgen_test]
fn packs_with_the_selected_algorithm() {
    let input = sizes(&[(2, 2), (2, 4)]);
    let value = pack_rects_js(input, "next-fit".into(), 10, 10).ok().unwrap();

    assert_eq!(field(&value, "name").as_string().unwrap(), "Next Fit");
    assert_eq!(field(&value, "usedArea").as_f64().unwrap(), 12.0);
    assert_eq!(field(&value, "sheetArea").as_f64().unwrap(), 16.0);
    assert_eq!(field(&value, "occupancy").as_f64().unwrap(), 0.75);

    let result: PackingResult = serde_wasm_bindgen::from_value(value).unwrap();
    assert_eq!(result.algorithm, PackingAlgorithm::NextFit);
    let placements = &result.sheets[0].placements;
    assert_eq!((placements[1].index, placements[1].x, placements[1].y), (1, 2, 0));
}

#[wasm_bindgen_test]
fn compares_all_algorithms() {
    let input = sizes(&[(64, 64); 16]);
    let value = compare_algorithms_js(input, 4096, 4096).ok().unwrap();
    assert_eq!(js_sys::Array::from(&value).length() as usize, PackingAlgorithm::ALL.len());
}

#[wasm_bindgen_test]
fn rejects_invalid_input() {
    assert!(pack_rects_js(sizes(&[(5, 1)]), "skyline".into(), 4, 4).is_err());
    assert!(pack_rects_js(sizes(&[(0, 1)]), "skyline".into(), 4, 4).is_err());
    assert!(pack_rects_js(sizes(&[(1, 1)]), "no-such-algorithm".into(), 4, 4).is_err());
    assert!(pack_rects_js("not an array".into(), "skyline".into(), 4, 4).is_err());
}
