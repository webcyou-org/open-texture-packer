//! Maximal Rectangles with the Best Short Side Fit rule: tracks every maximal
//! free rectangle, including overlapping ones, and places each sprite where
//! the shorter of its two leftover sides is smallest.

use super::Bin;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rect {
    fn right(&self) -> u32 {
        self.x + self.width
    }

    fn bottom(&self) -> u32 {
        self.y + self.height
    }

    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    fn contains(&self, other: &Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// The up to four maximal pieces of `self` left around `used`,
    /// which must intersect `self`.
    fn subtract(&self, used: &Rect, pieces: &mut Vec<Rect>) {
        if used.x > self.x {
            pieces.push(Rect {
                width: used.x - self.x,
                ..*self
            });
        }
        if used.right() < self.right() {
            pieces.push(Rect {
                x: used.right(),
                width: self.right() - used.right(),
                ..*self
            });
        }
        if used.y > self.y {
            pieces.push(Rect {
                height: used.y - self.y,
                ..*self
            });
        }
        if used.bottom() < self.bottom() {
            pieces.push(Rect {
                y: used.bottom(),
                height: self.bottom() - used.bottom(),
                ..*self
            });
        }
    }
}

pub(super) struct MaxRectsBin {
    free: Vec<Rect>,
}

impl MaxRectsBin {
    fn reserve(&mut self, used: &Rect) {
        let mut pieces = Vec::new();
        self.free.retain(|free| {
            if free.intersects(used) {
                free.subtract(used, &mut pieces);
                false
            } else {
                true
            }
        });

        // Keep only maximal rectangles. Untouched rectangles cannot contain
        // one another, so only comparisons involving a new piece are needed.
        let mut kept: Vec<Rect> = Vec::with_capacity(pieces.len());
        for (i, piece) in pieces.iter().enumerate() {
            let covered = self.free.iter().any(|free| free.contains(piece))
                || kept.iter().any(|other| other.contains(piece))
                || pieces[i + 1..].iter().any(|other| other.contains(piece));
            if !covered {
                kept.push(*piece);
            }
        }
        self.free.extend(kept);
    }
}

impl Bin for MaxRectsBin {
    fn new(width: u32, height: u32) -> Self {
        MaxRectsBin {
            free: vec![Rect {
                x: 0,
                y: 0,
                width,
                height,
            }],
        }
    }

    fn insert(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let mut best: Option<((u32, u32), u32, u32)> = None;

        for free in &self.free {
            if width > free.width || height > free.height {
                continue;
            }
            let leftover_w = free.width - width;
            let leftover_h = free.height - height;
            let score = (leftover_w.min(leftover_h), leftover_w.max(leftover_h));
            if best.is_none_or(|(best_score, _, _)| score < best_score) {
                best = Some((score, free.x, free.y));
            }
        }

        let (_, x, y) = best?;
        self.reserve(&Rect {
            x,
            y,
            width,
            height,
        });
        Some((x, y))
    }
}
