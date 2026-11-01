//! Shelf algorithms: sprites stand on horizontal shelves stacked upward from
//! the top of the sheet. Only the last shelf of a sheet can still grow taller.

use super::RawSheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShelfRule {
    /// Only the last shelf of the last sheet is considered.
    Next,
    /// The first shelf with room, on the first sheet with room.
    First,
    /// The shelf with the closest height, then the tightest remaining width.
    Best,
}

struct Shelf {
    y: u32,
    height: u32,
    used_width: u32,
}

struct ShelfSheet {
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
}

impl ShelfSheet {
    fn new(width: u32, height: u32) -> Self {
        ShelfSheet {
            width,
            height,
            shelves: Vec::new(),
        }
    }

    fn next_shelf_y(&self) -> u32 {
        self.shelves
            .last()
            .map_or(0, |shelf| shelf.y + shelf.height)
    }

    fn can_open_shelf(&self, height: u32) -> bool {
        height <= self.height - self.next_shelf_y()
    }

    fn fits_on_shelf(&self, index: usize, width: u32, height: u32) -> bool {
        let shelf = &self.shelves[index];
        let is_last = index + 1 == self.shelves.len();
        width <= self.width - shelf.used_width
            && (height <= shelf.height || (is_last && height <= self.height - shelf.y))
    }

    fn open_shelf(&mut self) -> usize {
        self.shelves.push(Shelf {
            y: self.next_shelf_y(),
            height: 0,
            used_width: 0,
        });
        self.shelves.len() - 1
    }

    fn place_on_shelf(&mut self, index: usize, width: u32, height: u32) -> (u32, u32) {
        let shelf = &mut self.shelves[index];
        let position = (shelf.used_width, shelf.y);
        shelf.used_width += width;
        shelf.height = shelf.height.max(height);
        position
    }
}

/// Where to place a rectangle on an existing sheet.
enum Target {
    Shelf { sheet: usize, shelf: usize },
    NewShelf { sheet: usize },
}

fn find_next(sheets: &[ShelfSheet], width: u32, height: u32) -> Option<Target> {
    let sheet_index = sheets.len().checked_sub(1)?;
    let sheet = &sheets[sheet_index];
    let last_shelf = sheet.shelves.len() - 1;

    if sheet.fits_on_shelf(last_shelf, width, height) {
        Some(Target::Shelf {
            sheet: sheet_index,
            shelf: last_shelf,
        })
    } else if sheet.can_open_shelf(height) {
        Some(Target::NewShelf { sheet: sheet_index })
    } else {
        None
    }
}

fn find_first(sheets: &[ShelfSheet], width: u32, height: u32) -> Option<Target> {
    for (sheet_index, sheet) in sheets.iter().enumerate() {
        for shelf in 0..sheet.shelves.len() {
            if sheet.fits_on_shelf(shelf, width, height) {
                return Some(Target::Shelf {
                    sheet: sheet_index,
                    shelf,
                });
            }
        }
        if sheet.can_open_shelf(height) {
            return Some(Target::NewShelf { sheet: sheet_index });
        }
    }
    None
}

fn find_best(sheets: &[ShelfSheet], width: u32, height: u32) -> Option<Target> {
    let mut best: Option<((u32, u32), usize, usize)> = None;

    for (sheet_index, sheet) in sheets.iter().enumerate() {
        for (shelf_index, shelf) in sheet.shelves.iter().enumerate() {
            if !sheet.fits_on_shelf(shelf_index, width, height) {
                continue;
            }
            let score = (
                shelf.height.abs_diff(height),
                sheet.width - shelf.used_width - width,
            );
            if best.is_none_or(|(best_score, _, _)| score < best_score) {
                best = Some((score, sheet_index, shelf_index));
            }
        }
    }

    if let Some((_, sheet, shelf)) = best {
        return Some(Target::Shelf { sheet, shelf });
    }

    sheets
        .iter()
        .position(|sheet| sheet.can_open_shelf(height))
        .map(|sheet| Target::NewShelf { sheet })
}

pub(super) fn pack(
    order: &[usize],
    sizes: &[(u32, u32)],
    rule: ShelfRule,
    bin_width: u32,
    bin_height: u32,
) -> Vec<RawSheet> {
    let mut sheets: Vec<ShelfSheet> = Vec::new();
    let mut placed: Vec<RawSheet> = Vec::new();

    for &index in order {
        let (width, height) = sizes[index];

        let target = match rule {
            ShelfRule::Next => find_next(&sheets, width, height),
            ShelfRule::First => find_first(&sheets, width, height),
            ShelfRule::Best => find_best(&sheets, width, height),
        };

        let (sheet, shelf) = match target {
            Some(Target::Shelf { sheet, shelf }) => (sheet, shelf),
            Some(Target::NewShelf { sheet }) => (sheet, sheets[sheet].open_shelf()),
            None => {
                let mut sheet = ShelfSheet::new(bin_width, bin_height);
                let shelf = sheet.open_shelf();
                sheets.push(sheet);
                placed.push(Vec::new());
                (sheets.len() - 1, shelf)
            }
        };

        let (x, y) = sheets[sheet].place_on_shelf(shelf, width, height);
        placed[sheet].push((index, x, y));
    }

    placed
}
