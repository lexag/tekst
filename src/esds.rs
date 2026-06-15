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
    Verdana15 = 0x40,
    Verdana16 = 0x50,
    TimesNewRoman16 = 0x51,
    Arial16 = 0x52,
    CourierNew16 = 0x53,
    System22 = 0x60,
    Arial22 = 0x61,
    ComicSans22 = 0x62,
    CourierNew22 = 0x63,
    TimesNewRoman22 = 0x64,
    Verdana32 = 0x90,
    Arial32 = 0x91,
    CourierNew32 = 0x93,
    TimesNewRoman32 = 0x94,
    LucidaConsole38 = 0xA0,
    Arial38 = 0xA1,
    ComicSans38 = 0xA2,
    CourierNew38 = 0xA3,
    TimesNewRoman38 = 0xA4,
    System48 = 0xB0,
    Arial48 = 0xB1,
    ComicSans48 = 0xB2,
    CourierNew48 = 0xB3,
    TimesNewRoman48 = 0xB4,
    LucidaConsole80 = 0xD0,
    Arial80 = 0xD1,
    ComicSans80 = 0xD2,
    CourierNew80 = 0xD3,
    TimesNewRoman80 = 0xD4,
}

impl Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Font::Verdana15 => write!(f, "[15] Verdana"),
            Font::Verdana16 => write!(f, "[16] Verdana"),
            Font::TimesNewRoman16 => write!(f, "[16] Times New Roman"),
            Font::Arial16 => write!(f, "[16] Arial"),
            Font::CourierNew16 => write!(f, "[16] Courier New"),
            Font::System22 => write!(f, "[22] System"),
            Font::Arial22 => write!(f, "[22] Arial"),
            Font::ComicSans22 => write!(f, "[22] Comic Sans"),
            Font::CourierNew22 => write!(f, "[22] Courier New"),
            Font::TimesNewRoman22 => write!(f, "[22] Times New Roman"),
            Font::Verdana32 => write!(f, "[32] Verdana"),
            Font::Arial32 => write!(f, "[32] Arial"),
            Font::CourierNew32 => write!(f, "[32] Courier New"),
            Font::TimesNewRoman32 => write!(f, "[32] Times New Roman"),
            Font::LucidaConsole38 => write!(f, "[38] Lucida Console"),
            Font::Arial38 => write!(f, "[38] Arial"),
            Font::ComicSans38 => write!(f, "[38] Comic Sans"),
            Font::CourierNew38 => write!(f, "[38] Courier New"),
            Font::TimesNewRoman38 => write!(f, "[38] Times New Roman"),
            Font::System48 => write!(f, "[48] System"),
            Font::Arial48 => write!(f, "[48] Arial"),
            Font::ComicSans48 => write!(f, "[48] Comic Sans"),
            Font::CourierNew48 => write!(f, "[48] Courier New"),
            Font::TimesNewRoman48 => write!(f, "[48] Times New Roman"),
            Font::LucidaConsole80 => write!(f, "[80] Lucida Console"),
            Font::Arial80 => write!(f, "[80] Arial"),
            Font::ComicSans80 => write!(f, "[80] Comic Sans"),
            Font::CourierNew80 => write!(f, "[80] Courier New"),
            Font::TimesNewRoman80 => write!(f, "[80] Times New Roman"),
        }
    }
}

impl InlineWidgetAutoEnum for Font {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![
            Font::Verdana15,
            Font::Verdana16,
            Font::TimesNewRoman16,
            Font::Arial16,
            Font::CourierNew16,
            Font::System22,
            Font::Arial22,
            Font::ComicSans22,
            Font::CourierNew22,
            Font::TimesNewRoman22,
            Font::Verdana32,
            Font::Arial32,
            Font::CourierNew32,
            Font::TimesNewRoman32,
            Font::LucidaConsole38,
            Font::Arial38,
            Font::ComicSans38,
            Font::CourierNew38,
            Font::TimesNewRoman38,
            Font::System48,
            Font::Arial48,
            Font::ComicSans48,
            Font::CourierNew48,
            Font::TimesNewRoman48,
            Font::LucidaConsole80,
            Font::Arial80,
            Font::ComicSans80,
            Font::CourierNew80,
            Font::TimesNewRoman80,
        ]
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
