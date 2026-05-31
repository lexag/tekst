use crate::{
    DISPLAY_NUM_LINES,
    esds::{Color, Font, TextAlign},
};

// fixme: impl actual type for this
type SMPTETimestamp = u8;

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
pub struct Cue {
    pub ident: String,
    pub text: [String; DISPLAY_NUM_LINES],
    pub brightness: Option<u8>,
    pub fade_speed: Option<u8>,
    pub text_color: Option<Color>,
    pub text_align: Option<TextAlign>,
    pub text_font: Option<Font>,
    pub autogo_delay_ms: Option<u16>,
    pub autogo_timecode: Option<SMPTETimestamp>,
    pub next_ident: Option<String>,
}

impl Cue {
    pub fn with_global_style(mut self, style: GlobalStyle) -> Self {
        self.brightness = self.brightness.or(Some(style.brightness));
        self.fade_speed = self.fade_speed.or(Some(style.fade_speed));
        self.text_color = self.text_color.or(Some(style.text_color));
        self.text_align = self.text_align.or(Some(style.text_align));
        self.text_font = self.text_font.or(Some(style.text_font));
        self
    }

    pub fn make_payload(&self) -> Vec<u8> {
        vec![]
    }
}

pub struct ImageCue {
    name: String,
    data: Vec<Color>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, Copy)]
pub struct GlobalStyle {
    pub brightness: u8,
    pub fade_speed: u8,
    pub text_color: Color,
    pub text_align: TextAlign,
    pub text_font: Font,
}
