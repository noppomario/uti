//! Selection and keys of the special item palette (no GTK types)

use super::list::Key;

/// What the UI must do after a palette key or click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteEffect {
    /// Show the rows of this category (its first row selected, if any).
    ShowCategory(usize),
    /// Highlight this row.
    Select(usize),
    /// Add the item of this row to the launcher.
    Add(usize),
    Close,
}

/// The open palette: a category tab and a row selected in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    /// Row count of each category.
    lens: Vec<usize>,
    category: usize,
    selected: Option<usize>,
}

impl Palette {
    /// Opens on `category` (clamped) with its first row selected.
    pub fn new(lens: Vec<usize>, category: usize) -> Self {
        let mut palette = Self {
            category: 0,
            selected: None,
            lens,
        };
        palette.set_category(category);
        palette
    }

    pub fn category(&self) -> usize {
        self.category
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// A tab was chosen: selects its first row.
    pub fn set_category(&mut self, category: usize) {
        self.category = category.min(self.lens.len().saturating_sub(1));
        self.selected = (self.len() > 0).then_some(0);
    }

    /// A row was clicked.
    pub fn select(&mut self, index: usize) {
        if index < self.len() {
            self.selected = Some(index);
        }
    }

    fn len(&self) -> usize {
        self.lens.get(self.category).copied().unwrap_or(0)
    }

    /// Left/Right switch categories, Up/Down move in the rows, Enter adds
    /// the selected row, Escape closes.
    pub fn on_key(&mut self, key: Key) -> Option<PaletteEffect> {
        match key {
            Key::Left | Key::Right => {
                let next = if key == Key::Left {
                    self.category.checked_sub(1)?
                } else {
                    self.category + 1
                };
                if next >= self.lens.len() {
                    return None;
                }
                self.set_category(next);
                Some(PaletteEffect::ShowCategory(next))
            },
            Key::Up | Key::Down => {
                let i = self.selected?;
                let next = if key == Key::Up {
                    i.checked_sub(1)?
                } else {
                    i + 1
                };
                (next < self.len()).then(|| {
                    self.selected = Some(next);
                    PaletteEffect::Select(next)
                })
            },
            Key::Enter => self.selected.map(PaletteEffect::Add),
            Key::Escape => Some(PaletteEffect::Close),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PaletteEffect::*;

    #[test]
    fn opens_on_a_category_with_its_first_row() {
        let p = Palette::new(vec![2, 3], 1);
        assert_eq!((p.category(), p.selected()), (1, Some(0)));
        let p = Palette::new(vec![2, 3], 7);
        assert_eq!(p.category(), 1);
        let p = Palette::new(vec![0], 0);
        assert_eq!(p.selected(), None);
    }

    #[test]
    fn left_and_right_switch_categories_without_wrapping() {
        let mut p = Palette::new(vec![2, 3, 1], 0);
        assert_eq!(p.on_key(Key::Left), None);
        p.on_key(Key::Down);
        assert_eq!(p.on_key(Key::Right), Some(ShowCategory(1)));
        assert_eq!(p.selected(), Some(0));
        assert_eq!(p.on_key(Key::Right), Some(ShowCategory(2)));
        assert_eq!(p.on_key(Key::Right), None);
        assert_eq!(p.on_key(Key::Left), Some(ShowCategory(1)));
    }

    #[test]
    fn up_and_down_stop_at_the_ends() {
        let mut p = Palette::new(vec![3], 0);
        assert_eq!(p.on_key(Key::Up), None);
        assert_eq!(p.on_key(Key::Down), Some(Select(1)));
        assert_eq!(p.on_key(Key::Down), Some(Select(2)));
        assert_eq!(p.on_key(Key::Down), None);
        assert_eq!(p.on_key(Key::Up), Some(Select(1)));
    }

    #[test]
    fn enter_adds_the_selected_row_and_escape_closes() {
        let mut p = Palette::new(vec![3], 0);
        p.select(2);
        assert_eq!(p.on_key(Key::Enter), Some(Add(2)));
        p.select(5);
        assert_eq!(p.selected(), Some(2));
        assert_eq!(p.on_key(Key::Escape), Some(Close));
        assert_eq!(p.on_key(Key::Digit(1)), None);
    }

    #[test]
    fn an_empty_category_has_nothing_to_add() {
        let mut p = Palette::new(vec![1, 0], 1);
        assert_eq!(p.on_key(Key::Down), None);
        assert_eq!(p.on_key(Key::Enter), None);
        assert_eq!(p.on_key(Key::Left), Some(ShowCategory(0)));
        assert_eq!(p.on_key(Key::Enter), Some(Add(0)));
    }
}
