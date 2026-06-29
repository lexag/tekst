use crate::{
    handler::State::Startup,
    receiver::Receiver,
    renderer::{DisplayBuffer, TextRenderer},
};
use std::time::Duration;
use tekst_common::{
    primitive::{Color, TextAlign, Transition},
    protocol::{DisplayContent, Message},
    textcontent::TextContent,
};

pub enum State {
    Idle,
    Startup(usize),
    Anim(usize),
}

pub struct Handler {
    pub receiver: Receiver,
    pub display: DisplayBuffer,
    pub renderer: TextRenderer,
    pub state: State,
}

impl Handler {
    const STARTUP_LEN: usize = 10;
    pub fn new() -> Self {
        Self {
            receiver: Receiver::new(),
            display: DisplayBuffer::new(),
            renderer: TextRenderer::new(),
            state: State::Startup(0),
        }
    }

    fn startup(&mut self, frame: usize) {
        match frame {
            Self::STARTUP_LEN => {}
            1 => {
                self.display = DisplayBuffer::test_pattern_a(Color::Green, 255);
            }
            2 => {
                self.display = DisplayBuffer::test_pattern_a(Color::Red, 255);
            }
            3 => {
                self.display = DisplayBuffer::test_pattern_a(Color::Amber, 255);
            }
            4 => self.display = DisplayBuffer::test_pattern_b(),
            5 => {
                self.display = self.renderer.render(TextContent {
                    text: vec![
                        "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG".to_string(),
                        "the quick brown fox jumps over the lazy dog".to_string(),
                    ],
                    brightness: 255,
                    transition: Transition::NoFade,
                    color: Color::Green,
                    align: TextAlign::Center,
                    font: tekst_common::primitive::Font::ComicSans22,
                });
            }
            6 => {
                self.display = self.renderer.render(TextContent {
                    text: vec![
                        "abcdefghijklmnopqrstuvwxyz".to_string(),
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
                    ],
                    brightness: 255,
                    transition: Transition::NoFade,
                    color: Color::Green,
                    align: TextAlign::Center,
                    font: tekst_common::primitive::Font::ComicSans22,
                });
            }
            8 => {
                self.display = self.renderer.render(TextContent {
                    text: vec![
                        "Connect:".to_string(),
                        match self.receiver.listener.local_addr() {
                            Ok(val) => val.to_string(),
                            Err(e) => format!("err: {e}"),
                        },
                    ],
                    brightness: 255,
                    transition: Transition::NoFade,
                    color: Color::Amber,
                    align: TextAlign::Left,
                    font: tekst_common::primitive::Font::ComicSans22,
                });
                self.state = State::Idle;
                return;
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_secs(1));
        self.state = State::Startup(frame + 1);
    }

    pub fn tick(&mut self) -> Option<DisplayBuffer> {
        match self.state {
            State::Idle => {
                let Some(msg) = self.receiver.rcv() else {
                    return None;
                };

                match msg {
                    Message::Show(content) => match content {
                        DisplayContent::Text(content) => {
                            self.display = self.renderer.render(content)
                        }
                        DisplayContent::Image(content) => {}
                        DisplayContent::Animation(content) => {}
                    },
                    Message::UploadImage(data, hash) => {}
                    Message::Response(_) => {}
                };
            }
            State::Startup(frame) => self.startup(frame),
            State::Anim(_) => todo!(),
        }
        Some(self.display)
    }
}
