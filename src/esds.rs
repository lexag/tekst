use crate::DISPLAY_NUM_LINES;
use egui::{Align, Color32, Response};
use std::{collections::HashMap, fmt::Display};

pub type Font = u8;

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

pub struct ESDSWrapper {
    pub text_mode: bool,
    text: [String; DISPLAY_NUM_LINES],
    image: Vec<Color>,
    pub color: Color,
    pub align: TextAlign,
    pub font: Font,
    pub brightness: u8,
    pub fade_speed: u8,
}

impl ESDSWrapper {
    pub fn new() -> Self {
        Self {
            text_mode: true,
            text: [const { String::new() }; DISPLAY_NUM_LINES],
            image: vec![],
            color: Color::Red,
            align: TextAlign::Left,
            font: 64,
            brightness: 255,
            fade_speed: 0,
        }
    }

    pub fn mut_text(&mut self, line: usize) -> &mut String {
        if line >= DISPLAY_NUM_LINES {
            &mut self.text[line]
        } else {
            &mut self.text[0]
        }
    }

    pub fn set_image(&mut self, image: Vec<Color>) {
        self.image = image;
    }
}
