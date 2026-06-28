use crate::{
    receiver::Receiver,
    renderer::{DisplayBuffer, TextRenderer},
};
use tekst_common::protocol::{DisplayContent, Message};

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
