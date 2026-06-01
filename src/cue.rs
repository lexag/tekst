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
        fn to_ahex(mut val: u8, num_bytes: usize) -> Vec<u8> {
            let mut out = vec![];
            while val > 0 {
                out.push((val & 0xF) + 0x30);
                val >>= 4;
            }
            out.resize(num_bytes, 0x30);
            out.reverse();
            out
        }

        let mut p = vec![];
        p.extend_from_slice(&[0x01, 0x31, 0x30, 0x30]);
        p.extend_from_slice(&[0x02, 0x80, 0x81, 0x1a]);
        p.extend_from_slice(&to_ahex(self.brightness.unwrap_or_default(), 2));
        p.extend_from_slice(&[0x00, 0x00]);
        p.extend_from_slice(&to_ahex(
            if self.fade_speed > Some(1) { 0xF } else { 0x1 },
            1,
        ));
        p.extend_from_slice(&to_ahex(self.fade_speed.unwrap_or_default(), 1));
        p.extend_from_slice(&[0x00, 0x00]);
        p.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        p.extend_from_slice(&[0x1b, 0x0e]);
        p.extend_from_slice(&to_ahex(self.text_font.unwrap_or_default(), 2));
        p.extend_from_slice(&to_ahex(self.text_color.unwrap_or_default() as u8, 1));
        p.extend_from_slice(&to_ahex(self.text_align.unwrap_or_default() as u8, 1));
        p.extend_from_slice(&[0x30]);
        p.extend_from_slice(&[0x30]);
        p.extend(&mut self.text[0].bytes());
        p.extend_from_slice(b"\r\n");
        p.extend(&mut self.text[1].bytes());
        p.extend_from_slice(&[0x00]);
        p.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        p
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
