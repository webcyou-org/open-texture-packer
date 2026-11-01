//! Skyline Bottom-Left: only the top contour of the placed rectangles is kept.
//! Each rectangle goes where its top edge stays lowest. Space hidden beneath
//! the contour is not reused, which makes this fast but less dense than
//! Maximal Rectangles.

use super::Bin;

#[derive(Debug, Clone, Copy)]
struct Segment {
    x: u32,
    y: u32,
    width: u32,
}

pub(super) struct SkylineBin {
    width: u32,
    height: u32,
    /// Covers `0..width` from left to right, without gaps.
    segments: Vec<Segment>,
}

impl SkylineBin {
    /// The `y` at which a rectangle starting on segment `start` would rest,
    /// or `None` if it would stick out of the bin.
    fn resting_y(&self, start: usize, width: u32, height: u32) -> Option<u32> {
        let x = self.segments[start].x;
        if width > self.width - x {
            return None;
        }

        let right = x + width;
        let y = self.segments[start..]
            .iter()
            .take_while(|segment| segment.x < right)
            .map(|segment| segment.y)
            .max()?;

        (height <= self.height - y).then_some(y)
    }

    /// Lays a rectangle spanning `x..x + width` onto the skyline.
    fn raise(&mut self, start: usize, width: u32, top: u32) {
        let x = self.segments[start].x;
        let right = x + width;

        let mut end = start;
        while end < self.segments.len() && self.segments[end].x < right {
            end += 1;
        }

        // The last covered segment may stick out on the right.
        let last = self.segments[end - 1];
        let last_right = last.x + last.width;

        let mut replacement = vec![Segment { x, y: top, width }];
        if last_right > right {
            replacement.push(Segment {
                x: right,
                y: last.y,
                width: last_right - right,
            });
        }
        self.segments.splice(start..end, replacement);

        self.merge_level_neighbours();
    }

    fn merge_level_neighbours(&mut self) {
        self.segments.dedup_by(|right, left| {
            let level = left.y == right.y;
            if level {
                left.width += right.width;
            }
            level
        });
    }
}

impl Bin for SkylineBin {
    fn new(width: u32, height: u32) -> Self {
        SkylineBin {
            width,
            height,
            segments: vec![Segment { x: 0, y: 0, width }],
        }
    }

    fn insert(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // (top edge, width of the segment it starts on), lowest first.
        let mut best: Option<((u32, u32), usize, u32)> = None;

        for start in 0..self.segments.len() {
            if let Some(y) = self.resting_y(start, width, height) {
                let score = (y + height, self.segments[start].width);
                if best.is_none_or(|(best_score, _, _)| score < best_score) {
                    best = Some((score, start, y));
                }
            }
        }

        let (_, start, y) = best?;
        let x = self.segments[start].x;
        self.raise(start, width, y + height);
        Some((x, y))
    }
}
