use crate::DISPLAY_NUM_LINES;
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
    pub mark: Option<String>,
    pub description: String,
    pub text: [String; DISPLAY_NUM_LINES],
    pub brightness: Option<u8>,
    pub fade_speed: Option<Transition>,
    pub text_color: Option<Color>,
    pub text_align: Option<TextAlign>,
    pub text_font: Option<Font>,
    pub autogo_delay_ms: Option<u16>,
    #[serde(
        serialize_with = "serialize_nested",
        deserialize_with = "deserialize_nested"
    )]
    pub autogo_timecode: Option<Timecode>,
    pub next_ident: Option<String>,
}

impl Cue {
    pub fn with_global_style(self, style: GlobalStyle) -> TextContent {
        TextContent {
            text: self.text.to_vec(),
            brightness: self
                .brightness
                .or(Some(style.brightness))
                .expect("or(Some())"),
            transition: self
                .fade_speed
                .or(Some(style.fade_speed))
                .expect("or(Some())"),
            color: self
                .text_color
                .or(Some(style.text_color))
                .expect("or(Some())"),
            align: self
                .text_align
                .or(Some(style.text_align))
                .expect("or(Some())"),
            font: self
                .text_font
                .or(Some(style.text_font))
                .expect("or(Some())"),
        }
    }
}

fn to_ahex(mut val: u8, num_bytes: usize) -> Vec<u8> {
    let mut out = vec![];
    while val > 0 {
        let digit = (val & 0xF);
        if digit <= 9 {
            out.push(digit + 0x30);
        } else {
            out.push(digit + 0x37);
        }
        val >>= 4;
    }
    out.resize(num_bytes, 0x30);
    out.reverse();
    out
}

pub struct ImageCue {
    name: String,
    data: Vec<Color>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, Copy)]
pub struct GlobalStyle {
    pub brightness: u8,
    pub fade_speed: Transition,
    pub text_color: Color,
    pub text_align: TextAlign,
    pub text_font: Font,
}

use ks_common_generic::smpte::Timecode;
use serde::{Deserialize, Deserializer, Serializer};
use tekst_common::{
    primitive::{Color, Font, TextAlign, Transition},
    textcontent::TextContent,
};

fn serialize_nested<S>(value: &Option<Timecode>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match value {
        Some(tc) => format!(
            "{}:{}:{}:{}:{}:{}:{}",
            tc.hours,
            tc.minutes,
            tc.seconds,
            tc.frames,
            tc.frame_rate.fps,
            tc.frame_rate.drop_frame,
            tc.user_bits
        ),
        None => String::new(),
    };

    serializer.serialize_str(&s)
}

fn deserialize_nested<'de, D>(deserializer: D) -> Result<Option<Timecode>, D::Error>
where
    D: Deserializer<'de>,
{
    fn tc_construct(mut comps: std::str::Split<'_, char>) -> Option<Timecode> {
        Some(Timecode::from_raw_fields(
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
            comps.next()?.parse().ok()?,
        ))
    }

    let s: String = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }

    let comps = s.split(':');

    match tc_construct(comps) {
        Some(tc) => Ok(Some(tc)),
        None => Err(serde::de::Error::custom("invalid timecode")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ahex() {
        assert_eq!(to_ahex(0x0, 2), [0x30, 0x30]);
        assert_eq!(to_ahex(0x5, 2), [0x30, 0x35]);
        assert_eq!(to_ahex(0xF, 2), [0x30, 0x46]);
        assert_eq!(to_ahex(0x32, 2), [0x33, 0x32]);
    }
}
