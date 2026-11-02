//! Rectangle packing engine.
//!
//! Works on plain `(width, height)` sizes, without decoded images, so that
//! GUI applications can preview and compare layouts cheaply.
//! [`crate::pack_sprites`] applies the result to loaded sprites.
//!
//! | Algorithm                               | Order      | Placement                                        |
//! |-----------------------------------------|------------|--------------------------------------------------|
//! | [`PackingAlgorithm::NextFit`]           | input      | current shelf only, never looks back             |
//! | [`PackingAlgorithm::ShelfPacking`]      | height     | current shelf only, never looks back             |
//! | [`PackingAlgorithm::FirstFit`]          | input      | first shelf with room                            |
//! | [`PackingAlgorithm::FirstFitDecreasing`]| height     | first shelf with room                            |
//! | [`PackingAlgorithm::BestFit`]           | input      | shelf with the closest height, then tightest gap |
//! | [`PackingAlgorithm::Skyline`]           | height     | lowest top edge on the skyline                   |
//! | [`PackingAlgorithm::MaximalRectangles`] | area       | free rectangle with the smallest short-side gap  |
//!
//! [`PackingAlgorithm::Skyline`] and [`PackingAlgorithm::MaximalRectangles`]
//! can also turn sprites by 90 degrees when [`PackOptions::allow_rotation`] is
//! set. The shelf algorithms keep every sprite upright.
//!
//! `NextFit` is the streaming algorithm: a single pass at the full sheet
//! width. All the others also try narrower sheet widths and keep the layout
//! with the fewest sheets and the smallest area, then refit the partly filled
//! last sheet on its own. Without that search, a set of sprites that fits
//! inside the maximum width would always end up as one long row, whichever
//! algorithm is selected.

mod maximal_rectangles;
mod shelf;
mod skyline;

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::error::Error;
use std::fmt;

use maximal_rectangles::MaxRectsBin;
use shelf::ShelfRule;
use skyline::SkylineBin;

/// Number of steps of the sheet width search.
const WIDTH_SEARCH_STEPS: u32 = 32;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum PackingAlgorithm {
    ShelfPacking,
    NextFit,
    BestFit,
    FirstFit,
    FirstFitDecreasing,
    Skyline,
    MaximalRectangles,
}

impl PackingAlgorithm {
    pub const ALL: [PackingAlgorithm; 7] = [
        PackingAlgorithm::ShelfPacking,
        PackingAlgorithm::NextFit,
        PackingAlgorithm::BestFit,
        PackingAlgorithm::FirstFit,
        PackingAlgorithm::FirstFitDecreasing,
        PackingAlgorithm::Skyline,
        PackingAlgorithm::MaximalRectangles,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PackingAlgorithm::ShelfPacking => "Shelf Packing",
            PackingAlgorithm::NextFit => "Next Fit",
            PackingAlgorithm::BestFit => "Best Fit",
            PackingAlgorithm::FirstFit => "First Fit",
            PackingAlgorithm::FirstFitDecreasing => "First Fit Decreasing",
            PackingAlgorithm::Skyline => "Skyline",
            PackingAlgorithm::MaximalRectangles => "Maximal Rectangles",
        }
    }

    /// Whether the algorithm uses [`PackOptions::allow_rotation`].
    pub fn supports_rotation(&self) -> bool {
        matches!(
            self,
            PackingAlgorithm::Skyline | PackingAlgorithm::MaximalRectangles
        )
    }
}

/// Settings that apply to every algorithm.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(default)]
pub struct PackOptions {
    /// Lets sprites turn by 90 degrees clockwise when that packs them better.
    /// Ignored by algorithms whose [`PackingAlgorithm::supports_rotation`] is
    /// false.
    pub allow_rotation: bool,
}

impl fmt::Display for PackingAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Position of one input rectangle in the result.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    /// Index in the slice given to [`pack_rects`].
    pub index: usize,
    pub x: u32,
    pub y: u32,
    /// Size on the sheet. For a rotated sprite this is the input size with
    /// width and height swapped.
    pub width: u32,
    pub height: u32,
    /// The sprite is turned 90 degrees clockwise: its top-left corner is at
    /// the top-right of the placement.
    #[serde(default)]
    pub rotated: bool,
}

/// One output sheet, cropped to its content.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PackedSheet {
    pub width: u32,
    pub height: u32,
    /// Sorted by input index, whatever order the algorithm placed them in.
    /// This keeps animation frames in order on every sheet.
    pub placements: Vec<Placement>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PackingResult {
    pub algorithm: PackingAlgorithm,
    pub sheets: Vec<PackedSheet>,
}

impl PackingResult {
    /// Pixels covered by the input rectangles.
    pub fn used_area(&self) -> u64 {
        self.sheets
            .iter()
            .flat_map(|sheet| &sheet.placements)
            .map(|p| p.width as u64 * p.height as u64)
            .sum()
    }

    /// Pixels of all output sheets.
    pub fn sheet_area(&self) -> u64 {
        self.sheets
            .iter()
            .map(|sheet| sheet.width as u64 * sheet.height as u64)
            .sum()
    }

    /// `used_area / sheet_area`, in the range `0.0..=1.0`.
    pub fn occupancy(&self) -> f64 {
        match self.sheet_area() {
            0 => 0.0,
            area => self.used_area() as f64 / area as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackingError {
    /// The rectangle has a width or height of zero.
    EmptySprite { index: usize },
    /// The rectangle does not fit in a sheet, even on its own.
    SpriteTooLarge {
        index: usize,
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
}

impl fmt::Display for PackingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackingError::EmptySprite { index } => {
                write!(f, "sprite #{index} has a width or height of zero")
            }
            PackingError::SpriteTooLarge {
                index,
                width,
                height,
                max_width,
                max_height,
            } => write!(
                f,
                "sprite #{index} ({width}x{height}) does not fit in the maximum sheet size ({max_width}x{max_height})"
            ),
        }
    }
}

impl Error for PackingError {}

/// Packs `(width, height)` sizes into as many sheets of at most
/// `max_width` x `max_height` as needed. Sprites stay upright.
pub fn pack_rects(
    sizes: &[(u32, u32)],
    algorithm: PackingAlgorithm,
    max_width: u32,
    max_height: u32,
) -> Result<PackingResult, PackingError> {
    pack_rects_with(sizes, algorithm, max_width, max_height, PackOptions::default())
}

/// [`pack_rects`] with options.
pub fn pack_rects_with(
    sizes: &[(u32, u32)],
    algorithm: PackingAlgorithm,
    max_width: u32,
    max_height: u32,
    options: PackOptions,
) -> Result<PackingResult, PackingError> {
    let rotate = options.allow_rotation && algorithm.supports_rotation();
    let bounds = Bounds {
        max_width,
        max_height,
        rotate,
    };
    validate(sizes, bounds)?;

    let sheets = if algorithm == PackingAlgorithm::NextFit {
        single_pass(sizes, algorithm, max_width, max_height, rotate)
    } else {
        let mut sheets = search_layout(sizes, algorithm, bounds);
        shrink_last_sheet(&mut sheets, algorithm, bounds);
        sheets
    };

    Ok(PackingResult { algorithm, sheets })
}

/// The sheet limits and whether sprites may turn.
#[derive(Debug, Clone, Copy)]
struct Bounds {
    max_width: u32,
    max_height: u32,
    rotate: bool,
}

/// The orientations to try for a sprite: upright first, then turned.
fn orientations(width: u32, height: u32, rotate: bool) -> impl Iterator<Item = (u32, u32, bool)> {
    let turned = (rotate && width != height).then_some((height, width, true));
    std::iter::once((width, height, false)).chain(turned)
}

/// Runs every algorithm on the same input, in the order of
/// [`PackingAlgorithm::ALL`].
pub fn compare_algorithms(
    sizes: &[(u32, u32)],
    max_width: u32,
    max_height: u32,
) -> Result<Vec<PackingResult>, PackingError> {
    compare_algorithms_with(sizes, max_width, max_height, PackOptions::default())
}

/// [`compare_algorithms`] with options.
pub fn compare_algorithms_with(
    sizes: &[(u32, u32)],
    max_width: u32,
    max_height: u32,
    options: PackOptions,
) -> Result<Vec<PackingResult>, PackingError> {
    PackingAlgorithm::ALL
        .iter()
        .map(|&algorithm| pack_rects_with(sizes, algorithm, max_width, max_height, options))
        .collect()
}

/// Tries every candidate sheet width and keeps the best layout.
fn search_layout(
    sizes: &[(u32, u32)],
    algorithm: PackingAlgorithm,
    bounds: Bounds,
) -> Vec<PackedSheet> {
    candidate_widths(sizes, bounds)
        .into_iter()
        .map(|width| single_pass(sizes, algorithm, width, bounds.max_height, bounds.rotate))
        .min_by_key(|sheets| layout_rank(sheets))
        .unwrap_or_default()
}

/// The search shares one bin size between all sheets, and earlier sheets need
/// it to be large. The partly filled last sheet can usually be much smaller.
fn shrink_last_sheet(sheets: &mut [PackedSheet], algorithm: PackingAlgorithm, bounds: Bounds) {
    let [_, .., last] = sheets else { return };

    // The input sizes, so that the refit can choose the orientation again.
    let sizes: Vec<(u32, u32)> = last
        .placements
        .iter()
        .map(|p| if p.rotated { (p.height, p.width) } else { (p.width, p.height) })
        .collect();
    let refit = search_layout(&sizes, algorithm, bounds);
    let [refit] = refit.as_slice() else { return };

    if refit.width as u64 * refit.height as u64 >= last.width as u64 * last.height as u64 {
        return;
    }

    // `refit` indexes into the last sheet and keeps its order.
    let placements = refit
        .placements
        .iter()
        .map(|p| Placement {
            index: last.placements[p.index].index,
            ..*p
        })
        .collect();
    *last = PackedSheet {
        width: refit.width,
        height: refit.height,
        placements,
    };
}

/// The narrowest width the sprite can take on a sheet of `bounds`, or `None`
/// if it fits in no orientation.
fn narrowest_width(width: u32, height: u32, bounds: Bounds) -> Option<u32> {
    orientations(width, height, bounds.rotate)
        .filter(|&(w, h, _)| w <= bounds.max_width && h <= bounds.max_height)
        .map(|(w, _, _)| w)
        .min()
}

fn validate(sizes: &[(u32, u32)], bounds: Bounds) -> Result<(), PackingError> {
    for (index, &(width, height)) in sizes.iter().enumerate() {
        if width == 0 || height == 0 {
            return Err(PackingError::EmptySprite { index });
        }
        if narrowest_width(width, height, bounds).is_none() {
            return Err(PackingError::SpriteTooLarge {
                index,
                width,
                height,
                max_width: bounds.max_width,
                max_height: bounds.max_height,
            });
        }
    }
    Ok(())
}

/// Sheet widths to try, spaced geometrically from the widest sprite up to the
/// width of a single row of all sprites (anything wider changes nothing).
/// With rotation, each sprite counts with its narrowest fitting width for the
/// lower end and its widest side for the row.
fn candidate_widths(sizes: &[(u32, u32)], bounds: Bounds) -> Vec<u32> {
    let lower = sizes
        .iter()
        .filter_map(|&(w, h)| narrowest_width(w, h, bounds))
        .max()
        .unwrap_or(0);
    let row: u64 = sizes
        .iter()
        .map(|&(w, h)| if bounds.rotate { w.max(h) } else { w } as u64)
        .sum();
    let upper = row.min(bounds.max_width as u64).max(lower as u64) as u32;
    if lower == 0 {
        return Vec::new();
    }

    let ratio = upper as f64 / lower as f64;
    let mut widths: Vec<u32> = (0..=WIDTH_SEARCH_STEPS)
        .map(|step| {
            let width = lower as f64 * ratio.powf(step as f64 / WIDTH_SEARCH_STEPS as f64);
            (width.round() as u32).clamp(lower, upper)
        })
        .collect();
    widths.dedup();
    widths
}

/// Lower is better: fewest sheets, then smallest area, then closest to square.
/// The width makes the choice deterministic when everything else is equal.
fn layout_rank(sheets: &[PackedSheet]) -> (usize, u64, u64, u32) {
    let area = sheets
        .iter()
        .map(|s| s.width as u64 * s.height as u64)
        .sum();
    let squareness = sheets
        .iter()
        .map(|s| s.width.abs_diff(s.height) as u64)
        .sum();
    let width = sheets.iter().map(|s| s.width).max().unwrap_or(0);
    (sheets.len(), area, squareness, width)
}

/// One pass of `algorithm` over bins of exactly `bin_width` x `bin_height`.
fn single_pass(
    sizes: &[(u32, u32)],
    algorithm: PackingAlgorithm,
    bin_width: u32,
    bin_height: u32,
    rotate: bool,
) -> Vec<PackedSheet> {
    use PackingAlgorithm::*;

    let order = match algorithm {
        NextFit | FirstFit | BestFit => (0..sizes.len()).collect(),
        ShelfPacking | FirstFitDecreasing | Skyline => {
            sorted_order(sizes, |w, h| (Reverse(h as u64), Reverse(w as u64)))
        }
        MaximalRectangles => sorted_order(sizes, |w, h| {
            (Reverse(w as u64 * h as u64), Reverse(w.max(h) as u64))
        }),
    };

    let raw = match algorithm {
        NextFit | ShelfPacking => {
            shelf::pack(&order, sizes, ShelfRule::Next, bin_width, bin_height)
        }
        FirstFit | FirstFitDecreasing => {
            shelf::pack(&order, sizes, ShelfRule::First, bin_width, bin_height)
        }
        BestFit => shelf::pack(&order, sizes, ShelfRule::Best, bin_width, bin_height),
        Skyline => pack_first_bin::<SkylineBin>(&order, sizes, bin_width, bin_height, rotate),
        MaximalRectangles => {
            pack_first_bin::<MaxRectsBin>(&order, sizes, bin_width, bin_height, rotate)
        }
    };

    raw.into_iter()
        .map(|placed| finish_sheet(placed, sizes))
        .collect()
}

/// Input indices sorted by `key`. The sort is stable, so ties keep input order.
fn sorted_order<K: Ord>(sizes: &[(u32, u32)], key: impl Fn(u32, u32) -> K) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| key(sizes[i].0, sizes[i].1));
    order
}

/// `(input index, x, y, rotated)` of the rectangles on one sheet.
type RawSheet = Vec<(usize, u32, u32, bool)>;

/// A single sheet being filled by a free-space algorithm.
trait Bin {
    fn new(width: u32, height: u32) -> Self;
    /// Returns the position and whether the rectangle was turned. It is only
    /// turned when `rotate` is set.
    fn insert(&mut self, width: u32, height: u32, rotate: bool) -> Option<(u32, u32, bool)>;
}

/// Puts each rectangle on the earliest sheet that has room for it.
fn pack_first_bin<B: Bin>(
    order: &[usize],
    sizes: &[(u32, u32)],
    bin_width: u32,
    bin_height: u32,
    rotate: bool,
) -> Vec<RawSheet> {
    let mut bins: Vec<B> = Vec::new();
    let mut sheets: Vec<RawSheet> = Vec::new();

    'rects: for &index in order {
        let (width, height) = sizes[index];
        for (bin, sheet) in bins.iter_mut().zip(sheets.iter_mut()) {
            if let Some((x, y, rotated)) = bin.insert(width, height, rotate) {
                sheet.push((index, x, y, rotated));
                continue 'rects;
            }
        }

        let mut bin = B::new(bin_width, bin_height);
        let (x, y, rotated) = bin
            .insert(width, height, rotate)
            .expect("validated rectangles fit in an empty bin");
        bins.push(bin);
        sheets.push(vec![(index, x, y, rotated)]);
    }

    sheets
}

fn finish_sheet(mut placed: RawSheet, sizes: &[(u32, u32)]) -> PackedSheet {
    placed.sort_unstable_by_key(|&(index, ..)| index);

    let placements: Vec<Placement> = placed
        .into_iter()
        .map(|(index, x, y, rotated)| {
            let (width, height) = sizes[index];
            let (width, height) = if rotated { (height, width) } else { (width, height) };
            Placement {
                index,
                x,
                y,
                width,
                height,
                rotated,
            }
        })
        .collect();

    PackedSheet {
        width: placements.iter().map(|p| p.x + p.width).max().unwrap_or(0),
        height: placements.iter().map(|p| p.y + p.height).max().unwrap_or(0),
        placements,
    }
}

#[cfg(test)]
mod tests {
    use super::PackingAlgorithm::*;
    use super::*;

    /// Small deterministic generator, so that failures can be reproduced.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn range(&mut self, min: u32, max: u32) -> u32 {
            min + (self.next() % (max - min + 1) as u64) as u32
        }
    }

    fn random_sizes(seed: u64, count: usize, min: u32, max: u32) -> Vec<(u32, u32)> {
        let mut rng = Rng(seed);
        (0..count)
            .map(|_| (rng.range(min, max), rng.range(min, max)))
            .collect()
    }

    fn positions(sheet: &PackedSheet) -> Vec<(u32, u32)> {
        sheet.placements.iter().map(|p| (p.x, p.y)).collect()
    }

    fn single_sheet_positions(
        algorithm: PackingAlgorithm,
        sizes: &[(u32, u32)],
    ) -> Vec<(u32, u32)> {
        let sheets = single_pass(sizes, algorithm, 10, 10, false);
        assert_eq!(sheets.len(), 1);
        positions(&sheets[0])
    }

    fn assert_valid(result: &PackingResult, sizes: &[(u32, u32)], max_w: u32, max_h: u32) {
        let name = result.algorithm;
        let mut seen = vec![false; sizes.len()];

        for sheet in &result.sheets {
            assert!(!sheet.placements.is_empty(), "{name}: empty sheet");
            assert!(
                sheet.width <= max_w && sheet.height <= max_h,
                "{name}: sheet too large"
            );

            for pair in sheet.placements.windows(2) {
                assert!(pair[0].index < pair[1].index, "{name}: not in input order");
            }

            for (i, a) in sheet.placements.iter().enumerate() {
                let (w, h) = sizes[a.index];
                let expected = if a.rotated { (h, w) } else { (w, h) };
                assert_eq!((a.width, a.height), expected, "{name}: size changed");
                assert!(!seen[a.index], "{name}: #{} placed twice", a.index);
                seen[a.index] = true;
                assert!(
                    a.x + a.width <= sheet.width && a.y + a.height <= sheet.height,
                    "{name}: #{} is outside the sheet",
                    a.index
                );

                for b in &sheet.placements[i + 1..] {
                    let apart = a.x + a.width <= b.x
                        || b.x + b.width <= a.x
                        || a.y + a.height <= b.y
                        || b.y + b.height <= a.y;
                    assert!(apart, "{name}: #{} overlaps #{}", a.index, b.index);
                }
            }
        }

        assert!(seen.iter().all(|&s| s), "{name}: sprite missing");
        assert!(result.occupancy() <= 1.0, "{name}: occupancy above 100%");
    }

    #[test]
    fn next_fit_never_returns_to_an_earlier_shelf() {
        let sizes = [(6, 4), (6, 2), (4, 2)];
        assert_eq!(
            single_sheet_positions(NextFit, &sizes),
            [(0, 0), (0, 4), (6, 4)]
        );
    }

    #[test]
    fn first_fit_uses_the_first_shelf_with_room() {
        let sizes = [(6, 4), (6, 2), (4, 2)];
        assert_eq!(
            single_sheet_positions(FirstFit, &sizes),
            [(0, 0), (0, 4), (6, 0)]
        );
    }

    #[test]
    fn best_fit_prefers_the_shelf_with_the_closest_height() {
        let sizes = [(6, 4), (6, 2), (4, 2)];
        assert_eq!(
            single_sheet_positions(BestFit, &sizes),
            [(0, 0), (0, 4), (6, 4)]
        );

        // Same height mismatch on both shelves: the tighter gap wins.
        let sizes = [(5, 3), (7, 3), (3, 3)];
        assert_eq!(
            single_sheet_positions(BestFit, &sizes),
            [(0, 0), (0, 3), (7, 3)]
        );
    }

    #[test]
    fn shelf_packing_sorts_by_height_and_never_looks_back() {
        let sizes = [(4, 2), (6, 4), (6, 3)];
        assert_eq!(
            single_sheet_positions(ShelfPacking, &sizes),
            [(6, 4), (0, 0), (0, 4)]
        );
    }

    #[test]
    fn first_fit_decreasing_sorts_by_height_and_fills_earlier_shelves() {
        let sizes = [(4, 2), (6, 4), (6, 3)];
        assert_eq!(
            single_sheet_positions(FirstFitDecreasing, &sizes),
            [(6, 0), (0, 0), (0, 4)]
        );
    }

    #[test]
    fn skyline_places_at_the_lowest_top_edge() {
        let sizes = [(4, 6), (6, 2), (2, 2)];
        assert_eq!(
            single_sheet_positions(Skyline, &sizes),
            [(0, 0), (4, 0), (4, 2)]
        );
    }

    #[test]
    fn skyline_loses_the_gap_that_maximal_rectangles_reuses() {
        let mut skyline = SkylineBin::new(10, 10);
        let mut max_rects = MaxRectsBin::new(10, 10);
        for (w, h) in [(4, 2), (10, 2)] {
            assert_eq!(skyline.insert(w, h, false), max_rects.insert(w, h, false));
        }
        assert_eq!(skyline.insert(6, 2, false), Some((0, 4, false)));
        assert_eq!(max_rects.insert(6, 2, false), Some((4, 0, false)));
    }

    #[test]
    fn maximal_rectangles_tiles_a_perfect_fit() {
        let sizes = [(6, 6), (4, 10), (6, 4)];
        let sheets = single_pass(&sizes, MaximalRectangles, 10, 10, false);
        assert_eq!(positions(&sheets[0]), [(4, 0), (0, 0), (4, 6)]);
        assert_eq!((sheets[0].width, sheets[0].height), (10, 10));
    }

    #[test]
    fn free_space_bins_refuse_what_does_not_fit() {
        let mut skyline = SkylineBin::new(4, 4);
        assert_eq!(skyline.insert(5, 1, false), None);
        assert_eq!(skyline.insert(1, 5, false), None);
        assert_eq!(skyline.insert(4, 4, false), Some((0, 0, false)));
        assert_eq!(skyline.insert(1, 1, false), None);

        let mut max_rects = MaxRectsBin::new(4, 4);
        assert_eq!(max_rects.insert(5, 1, false), None);
        assert_eq!(max_rects.insert(1, 5, false), None);
        assert_eq!(max_rects.insert(4, 4, false), Some((0, 0, false)));
        assert_eq!(max_rects.insert(1, 1, false), None);
    }

    #[test]
    fn all_algorithms_produce_valid_layouts() {
        let cases: [(usize, u32, u32, u32, u32); 6] = [
            (1, 1, 64, 64, 64),        // a single sprite
            (40, 32, 32, 256, 256),    // identical frames, several sheets
            (200, 1, 64, 256, 256),    // mixed sizes, several sheets
            (200, 1, 128, 4096, 4096), // mixed sizes, one large sheet
            (100, 1, 16, 16, 16),      // sprites up to the full sheet size
            (100, 1, 300, 300, 40),    // very wide sheet, sizes clamped below
        ];

        for seed in 1..=5u64 {
            for (count, min, max, sheet_w, sheet_h) in cases {
                let sizes: Vec<_> = random_sizes(seed, count, min, max)
                    .into_iter()
                    .map(|(w, h)| (w.min(sheet_w), h.min(sheet_h)))
                    .collect();
                for result in compare_algorithms(&sizes, sheet_w, sheet_h).unwrap() {
                    assert_valid(&result, &sizes, sheet_w, sheet_h);
                }
            }
        }
    }

    #[test]
    fn extreme_sheet_sizes_do_not_overflow() {
        let sizes = [(u32::MAX, 1), (1, u32::MAX), (1, 1)];
        for result in compare_algorithms(&sizes, u32::MAX, u32::MAX).unwrap() {
            let total: usize = result.sheets.iter().map(|s| s.placements.len()).sum();
            assert_eq!(total, sizes.len(), "{}", result.algorithm);
        }
    }

    #[test]
    fn results_are_deterministic() {
        let sizes = random_sizes(9, 150, 1, 96);
        assert_eq!(
            compare_algorithms(&sizes, 512, 512).unwrap(),
            compare_algorithms(&sizes, 512, 512).unwrap()
        );
    }

    #[test]
    fn width_search_avoids_a_single_row() {
        let sizes = vec![(64, 64); 16];
        for algorithm in PackingAlgorithm::ALL {
            let result = pack_rects(&sizes, algorithm, 4096, 4096).unwrap();
            let sheet = &result.sheets[0];
            if algorithm == NextFit {
                assert_eq!((sheet.width, sheet.height), (1024, 64));
            } else {
                assert_eq!((sheet.width, sheet.height), (256, 256), "{algorithm}");
            }
        }
    }

    #[test]
    fn last_sheet_is_refit_on_its_own() {
        // 28 frames fit per sheet, so 48 frames leave a partly filled last sheet.
        let sizes = vec![(96, 54); 48];
        for algorithm in PackingAlgorithm::ALL {
            let result = pack_rects(&sizes, algorithm, 409, 409).unwrap();
            assert_eq!(result.sheets.len(), 2, "{algorithm}");
            // Without the refit, Maximal Rectangles stays at 85.7% here.
            assert!(result.occupancy() > 0.97, "{algorithm}");
            assert_valid(&result, &sizes, 409, 409);
        }
    }

    #[test]
    fn search_never_uses_more_sheets_than_the_full_width() {
        for seed in 1..=5u64 {
            let sizes = random_sizes(seed, 200, 8, 96);
            for algorithm in PackingAlgorithm::ALL {
                let result = pack_rects(&sizes, algorithm, 512, 512).unwrap();
                let full_width = single_pass(&sizes, algorithm, 512, 512, false);
                assert!(result.sheets.len() <= full_width.len(), "{algorithm}");
            }
        }
    }

    #[test]
    fn free_space_algorithms_beat_next_fit_on_mixed_sizes() {
        let sizes = random_sizes(3, 150, 8, 96);
        let next_fit = pack_rects(&sizes, NextFit, 1024, 1024).unwrap().occupancy();
        for algorithm in [Skyline, MaximalRectangles] {
            let occupancy = pack_rects(&sizes, algorithm, 1024, 1024)
                .unwrap()
                .occupancy();
            assert!(
                occupancy > next_fit,
                "{algorithm}: {occupancy} <= {next_fit}"
            );
        }
    }

    #[test]
    fn rejects_sprites_that_cannot_be_placed() {
        for algorithm in PackingAlgorithm::ALL {
            assert_eq!(
                pack_rects(&[(4, 4), (5, 4)], algorithm, 4, 4),
                Err(PackingError::SpriteTooLarge {
                    index: 1,
                    width: 5,
                    height: 4,
                    max_width: 4,
                    max_height: 4,
                })
            );
            assert_eq!(
                pack_rects(&[(4, 0)], algorithm, 4, 4),
                Err(PackingError::EmptySprite { index: 0 })
            );
        }
    }

    #[test]
    fn empty_input_gives_no_sheets() {
        for algorithm in PackingAlgorithm::ALL {
            let result = pack_rects(&[], algorithm, 4, 4).unwrap();
            assert!(result.sheets.is_empty());
            assert_eq!(result.occupancy(), 0.0);
        }
    }

    fn rotation() -> PackOptions {
        PackOptions {
            allow_rotation: true,
        }
    }

    #[test]
    fn rotation_is_off_by_default_and_changes_nothing_when_off() {
        let sizes = random_sizes(4, 120, 1, 96);
        for algorithm in PackingAlgorithm::ALL {
            let plain = pack_rects(&sizes, algorithm, 512, 512).unwrap();
            let with = pack_rects_with(&sizes, algorithm, 512, 512, PackOptions::default()).unwrap();
            assert_eq!(plain, with, "{algorithm}");
            assert!(plain.sheets.iter().flat_map(|s| &s.placements).all(|p| !p.rotated));
        }
    }

    #[test]
    fn rotated_layouts_are_valid() {
        for seed in 1..=5u64 {
            for (count, min, max, sheet_w, sheet_h) in
                [(150, 1, 96, 512, 512), (80, 4, 40, 128, 64), (60, 1, 30, 30, 200)]
            {
                let sizes: Vec<_> = random_sizes(seed, count, min, max)
                    .into_iter()
                    .map(|(w, h)| (w.min(sheet_w.min(sheet_h)), h.min(sheet_w.min(sheet_h))))
                    .collect();
                for result in compare_algorithms_with(&sizes, sheet_w, sheet_h, rotation()).unwrap() {
                    assert_valid(&result, &sizes, sheet_w, sheet_h);
                    if !result.algorithm.supports_rotation() {
                        assert!(result.sheets.iter().flat_map(|s| &s.placements).all(|p| !p.rotated));
                    }
                }
            }
        }
    }

    #[test]
    fn rotation_fits_sprites_that_only_fit_turned() {
        // 10x40 only fits a 40x10 sheet lying down.
        let sizes = [(10, 40), (40, 10)];
        for algorithm in [Skyline, MaximalRectangles] {
            assert!(pack_rects(&sizes, algorithm, 40, 10).is_err());
            let result = pack_rects_with(&sizes, algorithm, 40, 10, rotation()).unwrap();
            assert_valid(&result, &sizes, 40, 10);
            let first = result.sheets.iter().flat_map(|s| &s.placements).find(|p| p.index == 0).unwrap();
            assert!(first.rotated);
            assert_eq!((first.width, first.height), (40, 10));
        }
        // Shelf algorithms keep sprites upright and report the sprite instead.
        assert!(pack_rects_with(&sizes, FirstFit, 40, 10, rotation()).is_err());
    }

    #[test]
    fn rotation_packs_tall_sprites_into_fewer_sheets() {
        // Tall sprites on a wide sheet: lying down, two fit per sheet.
        let sizes = vec![(20, 60); 4];
        for algorithm in [Skyline, MaximalRectangles] {
            let upright = pack_rects(&sizes, algorithm, 60, 40).unwrap_err();
            assert!(matches!(upright, PackingError::SpriteTooLarge { .. }));
            let turned = pack_rects_with(&sizes, algorithm, 60, 40, rotation()).unwrap();
            assert_eq!(turned.sheets.len(), 2, "{algorithm}");
        }
        let sizes = vec![(30, 100); 6];
        for algorithm in [Skyline, MaximalRectangles] {
            let upright = pack_rects(&sizes, algorithm, 200, 100).unwrap();
            let turned = pack_rects_with(&sizes, algorithm, 200, 100, rotation()).unwrap();
            assert!(turned.sheets.len() <= upright.sheets.len(), "{algorithm}");
            assert_valid(&turned, &sizes, 200, 100);
        }
    }

    #[test]
    fn rotation_is_deterministic() {
        let sizes = random_sizes(11, 150, 1, 96);
        assert_eq!(
            compare_algorithms_with(&sizes, 512, 512, rotation()).unwrap(),
            compare_algorithms_with(&sizes, 512, 512, rotation()).unwrap()
        );
    }

    #[test]
    fn reports_area_statistics() {
        let result = pack_rects(&[(2, 2), (2, 4)], NextFit, 10, 10).unwrap();
        assert_eq!(result.used_area(), 12);
        assert_eq!(result.sheet_area(), 16);
        assert_eq!(result.occupancy(), 0.75);
    }
}
