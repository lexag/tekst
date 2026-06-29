use crate::{
    receiver::Receiver,
    renderer::{DisplayBuffer, TextRenderer},
};
use egui::CursorIcon::Text;
use tekst_common::{
    primitive::{Color, TextAlign, Transition},
    protocol::{DisplayContent, Message},
    textcontent::TextContent,
};

pub struct Handler {
    pub receiver: Receiver,
    pub display: DisplayBuffer,
    pub renderer: TextRenderer,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            receiver: Receiver::new(),
            display: DisplayBuffer::new(),
            renderer: TextRenderer::new(),
        }
        .startup()
    }

    fn startup(mut self) -> Self {
        self.display = self.renderer.render(TextContent {
            text: vec![
                "The quick brown fox jumps over the lazy dog".to_string(),
                "Hello World".to_string(),
            ],
            brightness: 255,
            transition: Transition::NoFade,
            color: Color::Green,
            align: TextAlign::Center,
            font: tekst_common::primitive::Font::ComicSans22,
        });

        self
    }

    pub fn tick(&mut self) {
        let Some(msg) = self.receiver.rcv() else {
            return;
        };

        match msg {
            Message::Show(content) => match content {
                DisplayContent::Text(content) => self.display = self.renderer.render(content),
                DisplayContent::Image(content) => {}
                DisplayContent::Animation(content) => {}
            },
            Message::UploadImage(data, hash) => {}
            Message::Response(_) => {}
        }
    }
}
