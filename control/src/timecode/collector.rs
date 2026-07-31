use crate::timecode::{
    LTCReaderError, RenderableTimecodeHypothesis, TimecodeHypothesis, audio::AudioLTCReader,
};
use ks_common_generic::smpte::{FrameRate, Timecode};
use ks_common_ui::{
    components::Popup,
    traits::{
        ConfigurationWidget, InlineWidget, InlineWidgetMenu, SubstitutedAutoInlineWidgetMenu,
    },
};

struct TimecodeCollectorSources(AudioLTCReader);

pub struct TimecodeCollector {
    sources: TimecodeCollectorSources,
    frame_rate: FrameRate,
    last_known_timecode: TimecodeHypothesis,
}

impl Default for TimecodeCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TimecodeCollector {
    pub fn new() -> Self {
        Self {
            sources: TimecodeCollectorSources(AudioLTCReader::new()),
            frame_rate: FrameRate::Fps25,
            last_known_timecode: TimecodeHypothesis::default(),
        }
    }

    pub fn confidence(&self) -> f32 {
        self.last_known_timecode.1
    }

    pub fn read_timecode(&self) -> Result<Option<Timecode>, LTCReaderError> {
        Ok(None)
    }

    pub fn update(&mut self) -> Result<(), LTCReaderError> {
        // - get tc hypotheses from all children
        // - pick the best one (highest confidence and same as previous)
        // - if no child has >50% confidence, pick None
        // - return it
        Ok(())
    }
}

impl SubstitutedAutoInlineWidgetMenu<RenderableTimecodeHypothesis> for TimecodeCollector {
    fn substitute(&self) -> RenderableTimecodeHypothesis {
        RenderableTimecodeHypothesis::from(self.last_known_timecode)
    }
}

impl ConfigurationWidget for TimecodeCollector {
    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        self.sources.0.auto_inline_widget_menu(ui, "Audio LTC");
    }
}
