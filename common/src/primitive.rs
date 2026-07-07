#[cfg(feature = "egui")]
use egui::{Align, Color32, Response};
#[cfg(feature = "egui")]
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
    Sans,
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Font::Sans => write!(f, "Sans Serif"),
        }
    }
}

#[cfg(feature = "egui")]
impl InlineWidgetAutoEnum for Font {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Font::Sans]
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
pub enum Transition {
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

impl Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transition::NoFade => write!(f, "No Fade"),
            Transition::Speed1 => write!(f, "A seconds"),
            Transition::Speed2 => write!(f, "B seconds"),
            Transition::Speed3 => write!(f, "C seconds"),
            Transition::Speed4 => write!(f, "D seconds"),
            Transition::Speed5 => write!(f, "E seconds"),
            Transition::Speed6 => write!(f, "F seconds"),
            Transition::Speed7 => write!(f, "G seconds"),
            Transition::Speed8 => write!(f, "H seconds"),
            Transition::Speed9 => write!(f, "I seconds"),
        }
    }
}

impl From<u8> for Transition {
    fn from(value: u8) -> Self {
        match value {
            1 => Transition::Speed1,
            2 => Transition::Speed2,
            3 => Transition::Speed3,
            4 => Transition::Speed4,
            5 => Transition::Speed5,
            6 => Transition::Speed6,
            7 => Transition::Speed7,
            8 => Transition::Speed8,
            9 => Transition::Speed9,
            _ => Transition::NoFade,
        }
    }
}

#[cfg(feature = "egui")]
impl InlineWidgetAutoEnum for Transition {
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
            Transition::NoFade => Some("0.0s".to_string()),
            Transition::Speed1 => Some("A.As".to_string()),
            Transition::Speed2 => Some("B.Bs".to_string()),
            Transition::Speed3 => Some("C.Cs".to_string()),
            Transition::Speed4 => Some("D.Ds".to_string()),
            Transition::Speed5 => Some("E.Es".to_string()),
            Transition::Speed6 => Some("F.Fs".to_string()),
            Transition::Speed7 => Some("G.Gs".to_string()),
            Transition::Speed8 => Some("H.Hs".to_string()),
            Transition::Speed9 => Some("I.Is".to_string()),
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
    Blank = 0b00,
    Red = 0b01,
    Green = 0b10,
    #[default]
    Amber = 0b11,
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
    pub fn r(&self) -> bool {
        *self == Color::Red || *self == Color::Amber
    }

    pub fn g(&self) -> bool {
        *self == Color::Green || *self == Color::Amber
    }

    #[cfg(feature = "egui")]
    pub fn to_egui_color(self) -> Color32 {
        match self {
            Self::Blank => Color32::GRAY,
            Self::Red => Color32::RED,
            Self::Green => Color32::GREEN,
            Self::Amber => Color32::ORANGE,
        }
    }

    #[cfg(feature = "egui")]
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

#[cfg(feature = "egui")]
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
    #[cfg(feature = "egui")]
    pub fn to_egui_align(self) -> Align {
        match self {
            Self::Center => Align::Center,
            Self::Left => Align::Min,
            Self::Right => Align::Max,
        }
    }

    #[cfg(feature = "egui")]
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

    #[cfg(feature = "rasterizer")]
    pub fn to_cosmic_align(self) -> cosmic_text::Align {
        match self {
            Self::Center => cosmic_text::Align::Center,
            Self::Left => cosmic_text::Align::Left,
            Self::Right => cosmic_text::Align::Right,
        }
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

#[cfg(feature = "egui")]
impl InlineWidgetAutoEnum for TextAlign {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::Left, Self::Center, Self::Right]
    }
}
