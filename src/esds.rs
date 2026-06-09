use crate::DISPLAY_NUM_LINES;
use egui::{Align, Color32, Response};
use ks_common_ui::autoenum::InlineWidgetAutoEnum;
use std::fmt::Display;

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]

pub enum Font {
    #[default]
    FontA = 0x0,
    FontB = 0x1,
    FontC = 0x2,
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Font::FontA => write!(f, "Font A"),
            Font::FontB => write!(f, "Font B"),
            Font::FontC => write!(f, "Font C"),
        }
    }
}

impl InlineWidgetAutoEnum for Font {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::FontA, Self::FontB, Self::FontC]
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum FadeSpeed {
    #[default]
    NoFade = 0x0,
    Speed1 = 0x1,
    Speed2 = 0x2,
    Speed3 = 0x3,
    Speed4 = 0x4,
    Speed5 = 0x5,
    Speed6 = 0x6,
    Speed7 = 0x7,
    Speed8 = 0x8,
    Speed9 = 0x9,
}

impl Display for FadeSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FadeSpeed::NoFade => write!(f, "No Fade"),
            FadeSpeed::Speed1 => write!(f, "A seconds"),
            FadeSpeed::Speed2 => write!(f, "B seconds"),
            FadeSpeed::Speed3 => write!(f, "C seconds"),
            FadeSpeed::Speed4 => write!(f, "D seconds"),
            FadeSpeed::Speed5 => write!(f, "E seconds"),
            FadeSpeed::Speed6 => write!(f, "F seconds"),
            FadeSpeed::Speed7 => write!(f, "G seconds"),
            FadeSpeed::Speed8 => write!(f, "H seconds"),
            FadeSpeed::Speed9 => write!(f, "I seconds"),
        }
    }
}

impl From<u8> for FadeSpeed {
    fn from(value: u8) -> Self {
        match value {
            1 => FadeSpeed::Speed1,
            2 => FadeSpeed::Speed2,
            3 => FadeSpeed::Speed3,
            4 => FadeSpeed::Speed4,
            5 => FadeSpeed::Speed5,
            6 => FadeSpeed::Speed6,
            7 => FadeSpeed::Speed7,
            8 => FadeSpeed::Speed8,
            9 => FadeSpeed::Speed9,
            _ => FadeSpeed::NoFade,
        }
    }
}

impl InlineWidgetAutoEnum for FadeSpeed {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![
            Self::NoFade,
            Self::Speed1,
            Self::Speed2,
            Self::Speed3,
            Self::Speed4,
            Self::Speed5,
            Self::Speed6,
            Self::Speed7,
            Self::Speed8,
            Self::Speed9,
        ]
    }

    fn text(&self) -> Option<String> {
        match self {
            FadeSpeed::NoFade => Some("0.0s".to_string()),
            FadeSpeed::Speed1 => Some("A.As".to_string()),
            FadeSpeed::Speed2 => Some("B.Bs".to_string()),
            FadeSpeed::Speed3 => Some("C.Cs".to_string()),
            FadeSpeed::Speed4 => Some("D.Ds".to_string()),
            FadeSpeed::Speed5 => Some("E.Es".to_string()),
            FadeSpeed::Speed6 => Some("F.Fs".to_string()),
            FadeSpeed::Speed7 => Some("G.Gs".to_string()),
            FadeSpeed::Speed8 => Some("H.Hs".to_string()),
            FadeSpeed::Speed9 => Some("I.Is".to_string()),
        }
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum Color {
    Blank = 0x1,
    Red = 0x2,
    Green = 0x3,
    #[default]
    Amber = 0x4,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blank => write!(f, "Blank"),
            Self::Red => write!(f, "Red"),
            Self::Green => write!(f, "Green"),
            Self::Amber => write!(f, "Amber"),
        }
    }
}

impl Color {
    pub fn to_egui_color(self) -> Color32 {
        match self {
            Self::Blank => Color32::GRAY,
            Self::Red => Color32::RED,
            Self::Green => Color32::GREEN,
            Self::Amber => Color32::ORANGE,
        }
    }

    pub fn ui_selector(&mut self, ui: &mut egui::Ui) -> Response {
        egui::ComboBox::new(ui.id().with("color_select"), "")
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::Red, "Red");
                ui.selectable_value(self, Self::Green, "Green");
                ui.selectable_value(self, Self::Amber, "Amber");
            })
            .response
    }
}

impl InlineWidgetAutoEnum for Color {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::Red, Self::Green, Self::Amber]
    }

    fn color(&self) -> Option<Color32> {
        Some(self.to_egui_color())
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum TextAlign {
    Left = 0x1,
    Right = 0x2,
    #[default]
    Center = 0x3,
}

impl TextAlign {
    pub fn to_egui_align(self) -> Align {
        match self {
            Self::Center => Align::Center,
            Self::Left => Align::Min,
            Self::Right => Align::Max,
        }
    }

    pub fn ui_selector(&mut self, ui: &mut egui::Ui) -> Response {
        egui::ComboBox::new(ui.id().with("align select"), "")
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::Left, "Left");
                ui.selectable_value(self, Self::Center, "Center");
                ui.selectable_value(self, Self::Right, "Right");
            })
            .response
    }
}

impl Display for TextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Center => write!(f, "Center"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}

impl InlineWidgetAutoEnum for TextAlign {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::Left, Self::Center, Self::Right]
    }
}
