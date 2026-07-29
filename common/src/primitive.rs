#[cfg(feature = "egui")]
use egui::{Align, Color32, Response};
#[cfg(feature = "egui")]
use ks_common_ui::traits::InlineWidgetAutoEnum;
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
    NoTransition,
    FadeFast,
    FadeMedium,
    FadeSlow,
    FadeVerySlow,
}

impl From<u8> for Transition {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::FadeFast,
            2 => Self::FadeMedium,
            3 => Self::FadeSlow,
            4 => Self::FadeVerySlow,
            _ => Self::NoTransition,
        }
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transition::NoTransition => write!(f, "No Transition"),
            Transition::FadeFast => write!(f, "Fast Fade"),
            Transition::FadeMedium => write!(f, "Medium Fade"),
            Transition::FadeSlow => write!(f, "Slow Fade"),
            Transition::FadeVerySlow => write!(f, "Very slow Fade"),
        }
    }
}

impl Transition {
    pub fn duration(&self) -> f32 {
        match self {
            Transition::NoTransition => 0.0,
            Transition::FadeFast => 0.5,
            Transition::FadeMedium => 1.0,
            Transition::FadeSlow => 2.0,
            Transition::FadeVerySlow => 5.0,
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
            Self::NoTransition,
            Self::FadeFast,
            Self::FadeMedium,
            Self::FadeSlow,
            Self::FadeVerySlow,
        ]
    }

    fn text(&self) -> Option<String> {
        match self {
            Transition::NoTransition => Some("None".to_string()),
            Transition::FadeFast => Some("FastF".to_string()),
            Transition::FadeMedium => Some("MedmF".to_string()),
            Transition::FadeSlow => Some("SlowF".to_string()),
            Transition::FadeVerySlow => Some("VSlwF".to_string()),
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
