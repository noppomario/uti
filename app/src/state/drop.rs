//! Where a dragged row would land in a list (no GTK types)

/// Fraction of a folder row's height at its top and bottom that drops
/// before or after it; the middle drops into it.
const FOLDER_EDGE: f64 = 0.25;
/// Pixels at the top and bottom of a scrolled list that scroll it.
pub const SCROLL_EDGE: f64 = 24.0;
/// Pixels scrolled per tick with the pointer at the very edge.
const SCROLL_STEP: f64 = 12.0;

/// A drop position in a list of `len` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// Between rows, before row `index`.
    Before(usize),
    /// Into the folder at row `index`.
    Into(usize),
    /// After the last row.
    End,
}

/// The row under the pointer and the pointer's height within it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub index: usize,
    pub y: f64,
    pub height: f64,
}

/// Where the insertion line is drawn: at the top of `row`, or at its
/// bottom for `below` (the end of the list).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub row: usize,
    pub below: bool,
}

/// The spot for the pointer over `hit` in a list of `len` rows (`None`:
/// below the rows or an empty list); `folder` says which rows take drops.
pub fn spot(len: usize, folder: impl Fn(usize) -> bool, hit: Option<Hit>) -> Spot {
    let Some(hit) = hit.filter(|h| h.index < len) else {
        return Spot::End;
    };
    let f = hit.y / hit.height.max(1.0);
    let after = if folder(hit.index) {
        if f < FOLDER_EDGE {
            false
        } else if f > 1.0 - FOLDER_EDGE {
            true
        } else {
            return Spot::Into(hit.index);
        }
    } else {
        f >= 0.5
    };
    match (after, hit.index + 1 < len) {
        (false, _) => Spot::Before(hit.index),
        (true, true) => Spot::Before(hit.index + 1),
        (true, false) => Spot::End,
    }
}

impl Spot {
    /// The insertion line for this spot in a list of `len` rows; `None` for
    /// a folder (highlighted instead) or an empty list.
    pub fn line(self, len: usize) -> Option<Line> {
        match self {
            Spot::Before(row) => Some(Line { row, below: false }),
            Spot::End => len.checked_sub(1).map(|row| Line { row, below: true }),
            Spot::Into(_) => None,
        }
    }
}

/// Pixels to scroll for the pointer at `y` in a viewport `height` high:
/// negative near the top, positive near the bottom, faster at the edge.
pub fn autoscroll(y: f64, height: f64) -> f64 {
    let speed = |d: f64| SCROLL_STEP * (1.0 - d.max(0.0) / SCROLL_EDGE);
    if y < SCROLL_EDGE {
        -speed(y)
    } else if y > height - SCROLL_EDGE {
        speed(height - y)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(index: usize, y: f64) -> Option<Hit> {
        Some(Hit {
            index,
            y,
            height: 20.0,
        })
    }

    #[test]
    fn halves_of_a_row_drop_before_or_after_it() {
        let none = |_| false;
        assert_eq!(spot(3, none, hit(1, 4.0)), Spot::Before(1));
        assert_eq!(spot(3, none, hit(1, 10.0)), Spot::Before(2), "after row 1 is before row 2");
        assert_eq!(spot(3, none, hit(2, 15.0)), Spot::End);
    }

    #[test]
    fn the_middle_of_a_folder_drops_into_it() {
        let folder = |i| i == 1;
        assert_eq!(spot(3, folder, hit(1, 4.0)), Spot::Before(1));
        assert_eq!(spot(3, folder, hit(1, 6.0)), Spot::Into(1));
        assert_eq!(spot(3, folder, hit(1, 14.0)), Spot::Into(1));
        assert_eq!(spot(3, folder, hit(1, 16.0)), Spot::Before(2));
    }

    #[test]
    fn below_the_rows_or_an_empty_list_is_the_end() {
        assert_eq!(spot(2, |_| true, None), Spot::End);
        assert_eq!(spot(0, |_| true, hit(0, 10.0)), Spot::End, "the empty-level row");
    }

    #[test]
    fn lines_sit_above_the_row_or_below_the_last() {
        assert_eq!(
            Spot::Before(2).line(3),
            Some(Line {
                row: 2,
                below: false
            })
        );
        assert_eq!(
            Spot::End.line(3),
            Some(Line {
                row: 2,
                below: true
            })
        );
        assert_eq!(Spot::End.line(0), None);
        assert_eq!(Spot::Into(1).line(3), None);
    }

    #[test]
    fn edges_scroll_faster_closer_in() {
        assert_eq!(autoscroll(100.0, 300.0), 0.0);
        assert!(autoscroll(0.0, 300.0) < autoscroll(12.0, 300.0));
        assert!(autoscroll(12.0, 300.0) < 0.0);
        assert_eq!(autoscroll(300.0, 300.0), SCROLL_STEP);
        assert!(autoscroll(290.0, 300.0) > 0.0);
    }
}
