use crate::primitive::{Color, Font, TextAlign, Transition};
#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Hash,
)]
pub struct TextContent {
    pub text: Vec<String>,
    pub brightness: u8,
    pub transition: Transition,
    pub color: Color,
    pub align: TextAlign,
    pub font: Font,
}

impl TextContent {
    pub fn is_blank(&self) -> bool {
        self.text.is_empty() || self.text.iter().all(|s| s.is_empty())
    }
}
