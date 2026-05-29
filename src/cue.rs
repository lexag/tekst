use crate::{
    esds::{Color, Font, TextAlign},
    DISPLAY_NUM_LINES,
};

// fixme: impl actual type for this
type SMPTETimestamp = u8;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
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

pub struct ImageCue {
    name: String,
    data: Vec<Color>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct GlobalStyle {
    pub brightness: u8,
    pub fade_speed: u8,
    pub text_color: Color,
    pub text_align: TextAlign,
    pub text_font: Font,
}
