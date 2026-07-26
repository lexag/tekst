use crate::{
    handler::State::Startup,
    receiver::Receiver,
    renderer::{DisplayBuffer, TextRenderer},
};
use std::time::{Duration, Instant};
use tekst_common::{
    primitive::{Color, TextAlign, Transition},
    protocol::{DisplayContent, Message},
    textcontent::TextContent,
};

pub enum State {
    Idle,
    InTransitCountdown(TextContent, f32),
    Startup(usize),
    Anim(usize),
}

pub struct Handler {
    pub receiver: Receiver,
    pub display: DisplayBuffer,
    pub current_content: TextContent,
    pub renderer: TextRenderer,
    pub state: State,
    pub time_last_tick: Instant,
}

impl Handler {
    const STARTUP_LEN: usize = 10;
    pub fn new() -> Self {
        Self {
            receiver: Receiver::new(),
            current_content: TextContent::default(),
            display: DisplayBuffer::new(),
            renderer: TextRenderer::new(),
            time_last_tick: Instant::now(),
            #[cfg(feature = "startup-checks")]
            state: State::Startup(0),
            #[cfg(not(feature = "startup-checks"))]
            state: State::Idle,
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
                    transition: Transition::NoTransition,
                    color: Color::Green,
                    align: TextAlign::Center,
                    font: tekst_common::primitive::Font::Sans,
                });
            }
            6 => {
                self.display = self.renderer.render(TextContent {
                    text: vec![
                        "abcdefghijklmnopqrstuvwxyz".to_string(),
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
                    ],
                    brightness: 255,
                    transition: Transition::NoTransition,
                    color: Color::Green,
                    align: TextAlign::Center,
                    font: tekst_common::primitive::Font::Sans,
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
                    transition: Transition::NoTransition,
                    color: Color::Amber,
                    align: TextAlign::Left,
                    font: tekst_common::primitive::Font::Sans,
                });
                self.state = State::Idle;
                return;
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_secs(1));
        self.state = State::Startup(frame + 1);
    }

    pub fn tick<F>(&mut self, mut send_closure: F)
    where
        F: FnMut(DisplayBuffer),
    {
        match &self.state {
            State::Idle => {
                let Some(msg) = self.receiver.rcv() else {
                    return;
                };

                match msg {
                    Message::Show(content) => match content {
                        DisplayContent::Text(content) => {
                            if content.transition == Transition::NoTransition
                                || self.current_content.is_blank()
                            {
                                self.current_content = content;
                                self.display = self.renderer.render(self.current_content.clone());
                                (send_closure)(self.display);
                            } else {
                                self.display.set_animation(
                                    self.current_content.brightness,
                                    false,
                                    content.transition.duration(),
                                );
                                (send_closure)(self.display);
                                self.state = State::InTransitCountdown(
                                    content.clone(),
                                    content.transition.duration() * 1.8,
                                );
                            }
                        }
                        DisplayContent::Image(content) => {}
                        DisplayContent::Animation(content) => {}
                    },
                    Message::UploadImage(data, hash) => {}
                    Message::Response(_) => {}
                };
            }
            State::Startup(frame) => self.startup(*frame),
            State::Anim(_) => todo!(),
            State::InTransitCountdown(content, time_left) => {
                let delta = Instant::now()
                    .duration_since(self.time_last_tick)
                    .as_secs_f32();
                if *time_left < delta {
                    self.current_content = content.clone();
                    self.display = self.renderer.render(self.current_content.clone());
                    //            self.display.set_animation(
                    //                content.brightness,
                    //                true,
                    //                content.transition.duration(),
                    //            );
                    (send_closure)(self.display);
                    self.state = State::Idle;
                } else {
                    self.state = State::InTransitCountdown(content.clone(), time_left - delta);
                }
            }
        }
        self.time_last_tick = Instant::now();
    }
}
