use crate::timecode::{
    LTCReaderError, RenderableTimecodeHypothesis, TimecodeHypothesis, audio::AudioLTCReader,
    clicks::ClicksLTCReader, timer::TimerLTCReader,
};
use egui::Widget;
use ks_common_generic::smpte::{FrameRate, Timecode, TimecodeOffset, ltc::TimecodeReader};
use ks_common_ui::{
    components, material_icons, style,
    traits::{
        AutoInlineWidgetMenu, ConfigurationWidget, InlineWidget, SubstitutedAutoInlineWidgetMenu,
    },
};

pub struct TimecodeCollector {
    sources: (AudioLTCReader, ClicksLTCReader, TimerLTCReader),
    conversion_frame_rate: FrameRate,
    conversion_copy_input: bool,
    offset: TimecodeOffset,
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
            sources: (
                AudioLTCReader::new(),
                ClicksLTCReader::new(),
                TimerLTCReader::new(),
            ),
            offset: TimecodeOffset::default(),
            conversion_frame_rate: FrameRate::Fps25,
            conversion_copy_input: false,
            last_known_timecode: TimecodeHypothesis::default(),
        }
    }

    pub fn confidence(&self) -> f32 {
        self.last_known_timecode.1
    }

    pub fn read_timecode(&self) -> Result<Option<Timecode>, LTCReaderError> {
        if self.last_known_timecode.1 > 0.2 {
            Ok(Some(self.last_known_timecode.0))
        } else {
            Ok(None)
        }
    }

    pub fn update(&mut self) -> Result<(), LTCReaderError> {
        let results = [
            self.sources.0.read_timecode_confidence(),
            self.sources.1.read_timecode_confidence(),
            self.sources.2.read_timecode_confidence(),
        ];

        let mut record = TimecodeHypothesis::default();
        let mut possible_err = Ok(());
        for result in results {
            match result {
                Ok(h) => {
                    if h.1 > record.1 {
                        record = h;
                    }
                }
                Err(e) => possible_err = Err(e),
            }
        }

        self.last_known_timecode = ((record.0 + self.offset)?, record.1);

        if self.conversion_copy_input {
            self.conversion_frame_rate = FrameRate::from(self.last_known_timecode.0.frame_rate);
        }

        possible_err
    }
}

impl SubstitutedAutoInlineWidgetMenu<RenderableTimecodeHypothesis> for TimecodeCollector {
    fn substitute(&self) -> RenderableTimecodeHypothesis {
        RenderableTimecodeHypothesis::from(self.last_known_timecode)
    }
}

impl ConfigurationWidget for TimecodeCollector {
    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.sources
                    .0
                    .listening_thread
                    .is_some()
                    .inline_widget(ui, "On");
                self.sources.0.auto_inline_widget_menu(ui, "Audio LTC");
            });

            ui.horizontal(|ui| {
                self.sources
                    .1
                    .listening_thread
                    .is_some()
                    .inline_widget(ui, "On");
                self.sources.1.auto_inline_widget_menu(ui, "ClicKS LTC");
            });

            ui.horizontal(|ui| {
                if components::Button::new("Play")
                    .icon(material_icons::Icon::PlayArrow)
                    .indicator(self.sources.2.running().then_some(style::ACTIVE_COLOR))
                    .ui(ui)
                    .clicked()
                {
                    self.sources.2.toggle();
                }
                self.sources.2.auto_inline_widget_menu(ui, "Timer LTC");
            });

            self.offset.auto_inline_widget_menu(ui, "Offset");

            ui.horizontal(|ui| {
                self.conversion_frame_rate
                    .auto_inline_widget_menu(ui, "Frame rate (conv)");

                components::ToggleButton::new(
                    &mut self.conversion_copy_input,
                    material_icons::Icon::AutofpsSelect,
                    style::ACCENT_COLOR,
                )
                .ui(ui);
            });
        });
    }
}
