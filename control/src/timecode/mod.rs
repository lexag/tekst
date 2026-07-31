use crate::timecode::audio::AudioLTCReader;
use egui::Widget;
use ks_common_generic::smpte::{Timecode, ltc::TimecodeReader};
use ks_common_ui::{
    style,
    traits::{
        ConfigurationWidget, InlineWidget, InlineWidgetMenu, SubstitutedAutoInlineWidgetMenu,
    },
};
use std::{any::Any, error::Error, fmt::Display};

pub mod audio;
pub mod clicks;
pub mod collector;
pub mod old;
pub mod timer;

type TimecodeHypothesis = (Timecode, f32);

// FIXME: this is annoying...
// `impl InlineWidget for TimecodeHypothesis` does not work straight on the tuple, so we have to
// wrap it... Should really be done some other way that I can't be bothered with right now :/
pub struct RenderableTimecodeHypothesis(Timecode, f32);

impl From<TimecodeHypothesis> for RenderableTimecodeHypothesis {
    fn from(value: TimecodeHypothesis) -> Self {
        Self(value.0, value.1)
    }
}

impl InlineWidget for RenderableTimecodeHypothesis {
    fn draw(&mut self, ui: &mut egui::Ui, label: &str) -> egui::Response {
        let s = &self.0.to_string();
        if self.1 <= 0.0 {
            ks_common_ui::components::TextDisplay::new("NO TC", 11)
                .color(ui.visuals().warn_fg_color)
        } else {
            ks_common_ui::components::TextDisplay::new(s, 11).color(match self.1 {
                0.3..=0.7 => ui.visuals().warn_fg_color,
                0.7.. => style::ACTIVE_COLOR,
                _ => ui.visuals().error_fg_color,
            })
        }
        .label(label)
        .align(egui::Align::Center)
        .ui(ui)
    }
}

#[derive(Debug)]
pub enum LTCReaderError {
    Timecode(ks_common_generic::smpte::TimecodeError),
    RodioMicrophoneOpen(rodio::microphone::OpenError),
    RodioMicrophoneList(rodio::microphone::ListError),
    RodioInternal,
    NoSelection,
    Anonymous(Box<dyn Any + Send>),
    Unknown,
}

impl Display for LTCReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unimplemented, see src/timecode/mod.rs")
    }
}

impl Error for LTCReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl From<rodio::microphone::OpenError> for LTCReaderError {
    fn from(value: rodio::microphone::OpenError) -> Self {
        Self::RodioMicrophoneOpen(value)
    }
}

impl From<rodio::microphone::ListError> for LTCReaderError {
    fn from(value: rodio::microphone::ListError) -> Self {
        Self::RodioMicrophoneList(value)
    }
}

impl From<Box<dyn Any + Send>> for LTCReaderError {
    fn from(value: Box<dyn Any + Send>) -> Self {
        Self::Anonymous(value)
    }
}

impl From<ks_common_generic::smpte::TimecodeError> for LTCReaderError {
    fn from(value: ks_common_generic::smpte::TimecodeError) -> Self {
        Self::Timecode(value)
    }
}

trait TimecodeReaderWidget:
    TimecodeReader<LTCReaderError> + ConfigurationWidget + InlineWidgetMenu
{
}
