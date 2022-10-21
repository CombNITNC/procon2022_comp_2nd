#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardVoiceIndex(u8);

impl std::fmt::Display for CardVoiceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 44 {
            write!(f, "E{:02}", self.0 + 1)
        } else {
            write!(f, "J{:02}", self.0 + 1 - 44)
        }
    }
}

impl CardVoiceIndex {
    #[inline]
    pub fn new(index: u8) -> Self {
        assert!(index < 88);
        Self(index)
    }

    #[inline]
    pub fn all() -> impl Iterator<Item = CardVoiceIndex> {
        (0..88).map(Self::new)
    }

    #[inline]
    pub fn into_answer_string(self) -> String {
        if self.0 < 44 {
            (self.0 + 1).to_string()
        } else {
            (self.0 + 1 - 44).to_string()
        }
    }
}
