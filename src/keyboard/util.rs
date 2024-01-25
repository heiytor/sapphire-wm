#[derive(Hash, PartialEq, Eq)]
pub struct KeyCombination {
    pub keycode: u8,
    pub modifier: u16,
}
