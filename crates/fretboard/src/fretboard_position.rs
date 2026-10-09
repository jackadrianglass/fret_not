#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FretboardPosition {
    pub string_index: usize,
    pub fret: i32,
}
