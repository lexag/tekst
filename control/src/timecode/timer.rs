use crate::timecode::{LTCReaderError, TimecodeHypothesis};
use ks_common_generic::smpte::{
    FrameRate, Timecode, TimecodeError, ltc::TimecodeReader,
};
use ks_common_ui::traits::{
    ConfigurationWidget, SubstitutedAutoInlineWidgetMenu,
};
use std::time::Instant;

pub struct TimerLTCReader {
    last_seen_timecode: TimecodeHypothesis,

    start_tc: Timecode,
    start_instant: Option<Instant>,
}

impl TimecodeReader<LTCReaderError> for TimerLTCReader {
    fn read_timecode(
        &mut self,
    ) -> Result<Option<ks_common_generic::smpte::Timecode>, LTCReaderError> {
        let Some(start_time) = self.start_instant else {
            return Ok(None);
        };
        let dur = start_time.elapsed();

        self.last_seen_timecode = (
            (Timecode::from_frames(
                u64::try_from(dur.as_millis()).map_err(|_| TimecodeError::InvalidFrames)? / 20,
                FrameRate::Fps50,
            ) + self.start_tc),
            1.0,
        );

        Ok(Some(self.last_seen_timecode.0))
    }

    fn frame_rate(&self) -> ks_common_generic::smpte::FrameRate {
        FrameRate::Fps50
    }

    fn is_synchronized(&self) -> bool {
        self.running()
    }
}

impl TimerLTCReader {
    pub fn new() -> Self {
        Self {
            last_seen_timecode: TimecodeHypothesis::default(),
            start_tc: Timecode::default(),
            start_instant: None,
        }
    }

    pub fn start(&mut self) {
        self.start_instant = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.start_instant = None;
    }

    pub fn toggle(&mut self) {
        if self.running() {
            self.stop();
        } else {
            self.start();
        }
    }

    pub fn running(&self) -> bool {
        self.start_instant.is_some()
    }
}

impl SubstitutedAutoInlineWidgetMenu<Timecode> for TimerLTCReader {
    fn substitute(&self) -> Timecode {
        if self.running() {
            self.last_seen_timecode.0
        } else {
            self.start_tc
        }
    }
}

impl ConfigurationWidget for TimerLTCReader {
    fn draw_configuration(&mut self, ui: &mut egui::Ui) -> egui::Response {
        self.start_tc.draw_configuration(ui)
    }

    fn grid_contents(&mut self, _ui: &mut egui::Ui) {
        unimplemented!()
    }
}
