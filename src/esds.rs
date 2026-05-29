use std::collections::HashMap;

type Font = u8;

pub enum Color {
    Blank = 0x1,
    Red = 0x2,
    Green = 0x3,
    Amber = 0x4,
}

pub enum TextAlign {
    Left = 0x1,
    Right = 0x2,
    Center = 0x3,
}

pub struct ESDSWrapper {
    pub text_mode: bool,
    text: HashMap<usize, String>,
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
            text: HashMap::new(),
            image: vec![],
            color: Color::Red,
            align: TextAlign::Left,
            font: 64,
            brightness: 255,
            fade_speed: 0,
        }
    }

    pub fn mut_text(&mut self, line: usize) -> &mut String {
        self.text.entry(line).or_default()
    }

    pub fn set_image(&mut self, image: Vec<Color>) {
        self.image = image;
    }
}
