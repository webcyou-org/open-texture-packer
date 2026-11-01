//! JavaScript bindings for the packing engine.
//!
//! Only layouts are computed here. Images are decoded and drawn by the browser.

use open_texture_packer::{pack_rects, PackedSheet, PackingAlgorithm, PackingResult};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AlgorithmInfo {
    id: PackingAlgorithm,
    name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsPackingResult {
    algorithm: PackingAlgorithm,
    name: &'static str,
    sheets: Vec<PackedSheet>,
    used_area: f64,
    sheet_area: f64,
    occupancy: f64,
}

impl From<PackingResult> for JsPackingResult {
    fn from(result: PackingResult) -> Self {
        JsPackingResult {
            algorithm: result.algorithm,
            name: result.algorithm.name(),
            used_area: result.used_area() as f64,
            sheet_area: result.sheet_area() as f64,
            occupancy: result.occupancy(),
            sheets: result.sheets,
        }
    }
}

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsError::new(&error.to_string()))
}

fn read_sizes(sizes: JsValue) -> Result<Vec<(u32, u32)>, JsError> {
    let sizes: Vec<Size> =
        serde_wasm_bindgen::from_value(sizes).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(sizes.iter().map(|s| (s.width, s.height)).collect())
}

fn pack(
    sizes: &[(u32, u32)],
    algorithm: PackingAlgorithm,
    max_width: u32,
    max_height: u32,
) -> Result<JsPackingResult, JsError> {
    let result = pack_rects(sizes, algorithm, max_width, max_height)
        .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(result.into())
}

/// Lists the algorithms as `[{ id: "skyline", name: "Skyline" }, ...]`.
#[wasm_bindgen]
pub fn algorithms() -> Result<JsValue, JsError> {
    let list: Vec<AlgorithmInfo> = PackingAlgorithm::ALL
        .iter()
        .map(|&id| AlgorithmInfo {
            id,
            name: id.name(),
        })
        .collect();
    to_js(&list)
}

/// Packs `[{ width, height }, ...]` with one algorithm. `placements[].index`
/// refers to the position in `sizes`. Throws if a size cannot be placed.
#[wasm_bindgen(js_name = packRects)]
pub fn pack_rects_js(
    sizes: JsValue,
    algorithm: JsValue,
    #[wasm_bindgen(js_name = maxWidth)] max_width: u32,
    #[wasm_bindgen(js_name = maxHeight)] max_height: u32,
) -> Result<JsValue, JsError> {
    let sizes = read_sizes(sizes)?;
    let algorithm: PackingAlgorithm = serde_wasm_bindgen::from_value(algorithm)
        .map_err(|error| JsError::new(&error.to_string()))?;
    to_js(&pack(&sizes, algorithm, max_width, max_height)?)
}

/// Packs the same sizes with every algorithm.
#[wasm_bindgen(js_name = compareAlgorithms)]
pub fn compare_algorithms_js(
    sizes: JsValue,
    #[wasm_bindgen(js_name = maxWidth)] max_width: u32,
    #[wasm_bindgen(js_name = maxHeight)] max_height: u32,
) -> Result<JsValue, JsError> {
    let sizes = read_sizes(sizes)?;
    let results = PackingAlgorithm::ALL
        .iter()
        .map(|&algorithm| pack(&sizes, algorithm, max_width, max_height))
        .collect::<Result<Vec<_>, _>>()?;
    to_js(&results)
}
