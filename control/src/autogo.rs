use crate::{app::TekstApp, cue::Cue, timecode::collector::TimecodeCollector};
use egui::Context;
use ks_common_generic::smpte::ltc::TimecodeReader;
use ks_common_generic::smpte::{Timecode, TimecodeOffset};
use ks_common_ui::traits::InlineWidgetAutoEnum;
use std::{fmt::Display, ops::Sub};

pub trait AutoGo {
    fn time_until_go(&self, cue: &Cue) -> f64;
    fn max_time_until_go(&self, cue: &Cue) -> f64;

    fn progress(&self, cue: &Cue) -> f32 {
        1.0 - (self.time_until_go(cue) / self.max_time_until_go(cue)).clamp(0.0, 1.0) as f32
    }

    fn requests_go(&mut self, cue: &Cue) -> bool {
        if *self.mode_mut() != AutoGoOpMode::Hint {
            self.time_until_go(cue) <= 0.0
        } else {
            false
        }
    }

    fn mode_mut(&mut self) -> &mut AutoGoOpMode;

    fn go_happened(&mut self, cue: &mut Cue);
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum AutoGoOpMode {
    Off,
    Hint,
    Ctrl,
    Learn,
}

impl Display for AutoGoOpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoGoOpMode::Off => write!(f, "Off"),
            AutoGoOpMode::Hint => write!(f, "Hint"),
            AutoGoOpMode::Ctrl => write!(f, "Control"),
            AutoGoOpMode::Learn => write!(f, "Learn"),
        }
    }
}

impl InlineWidgetAutoEnum for AutoGoOpMode {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::Off, Self::Hint, Self::Ctrl, Self::Learn]
    }

    fn color(&self) -> Option<egui::Color32> {
        match self {
            AutoGoOpMode::Off => None,
            AutoGoOpMode::Hint => Some(ks_common_ui::style::ACCENT_COLOR),
            AutoGoOpMode::Ctrl => Some(ks_common_ui::style::ACTIVE_COLOR),
            AutoGoOpMode::Learn => Some(ks_common_ui::style::WARNING_COLOR),
        }
    }

    fn text(&self) -> Option<String> {
        match self {
            AutoGoOpMode::Off => Some("OFF".to_string()),
            AutoGoOpMode::Hint => Some("HNT".to_string()),
            AutoGoOpMode::Ctrl => Some("CTL".to_string()),
            AutoGoOpMode::Learn => Some("LRN".to_string()),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoGoConsolidator {
    pub timecode: AutoTimecode,
    pub follow: AutoFollow,
}
impl AutoGoConsolidator {
    pub fn new(ctx: Context) -> Self {
        Self {
            timecode: AutoTimecode::new(),
            follow: AutoFollow::new(ctx),
        }
    }

    pub fn time_until_go(&self, cue: &Cue) -> f64 {
        self.timecode
            .time_until_go(cue)
            .min(self.follow.time_until_go(cue))
    }

    pub fn max_time_until_go(&self, cue: &Cue) -> f64 {
        self.timecode
            .max_time_until_go(cue)
            .min(self.follow.max_time_until_go(cue))
    }

    pub fn progress(&self, cue: &Cue) -> f32 {
        let p = 1.0 - (self.time_until_go(cue) / self.max_time_until_go(cue)) as f32;
        if p.is_finite() { p } else { 0.0 }
    }

    pub fn requests_go(&mut self, cue: &Cue) -> bool {
        self.timecode.requests_go(cue) || self.follow.requests_go(cue)
    }

    pub fn go_happened(&mut self, mut cue: Cue) -> Cue {
        self.timecode.go_happened(&mut cue);
        self.follow.go_happened(&mut cue);
        cue
    }

    pub fn dry_go_happened(&mut self) {
        let _ = self.go_happened(Cue::default());
    }

    pub fn any_active(&self) -> bool {
        self.follow.mode != AutoGoOpMode::Off || self.timecode.mode != AutoGoOpMode::Off
    }
    pub fn any_learn(&self) -> bool {
        self.follow.mode == AutoGoOpMode::Learn || self.timecode.mode == AutoGoOpMode::Learn
    }
    pub fn any_hint(&self) -> bool {
        self.follow.mode == AutoGoOpMode::Hint || self.timecode.mode == AutoGoOpMode::Hint
    }

    pub fn toggle_learn(&mut self) {
        if self.timecode.mode == AutoGoOpMode::Learn {
            *self.follow.mode_mut() = AutoGoOpMode::Off;
            *self.timecode.mode_mut() = AutoGoOpMode::Off;
        } else if self.follow.mode == AutoGoOpMode::Learn {
            *self.follow.mode_mut() = AutoGoOpMode::Off;
            *self.timecode.mode_mut() = AutoGoOpMode::Learn;
        } else {
            *self.follow.mode_mut() = AutoGoOpMode::Learn;
            *self.timecode.mode_mut() = AutoGoOpMode::Off;
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoTimecode {
    #[serde(skip)]
    pub timecode_reader: TimecodeCollector,
    pub offset: TimecodeOffset,
    pub mode: AutoGoOpMode,
}

impl AutoGo for AutoTimecode {
    fn time_until_go(&self, cue: &Cue) -> f64 {
        if (self.mode == AutoGoOpMode::Ctrl || self.mode == AutoGoOpMode::Hint)
            && let Ok(Some(tc)) = self.timecode_reader.read_timecode()
            && let actual_tc = tc - self.offset
            && let Some(cue_tc) = cue.autogo_timecode
            && self.timecode_reader.confidence() > 0.75
        {
            return cue_tc.to_seconds_f64() - actual_tc.to_seconds_f64();
        }
        f64::INFINITY
    }

    fn mode_mut(&mut self) -> &mut AutoGoOpMode {
        &mut self.mode
    }

    fn go_happened(&mut self, cue: &mut Cue) {
        if self.mode == AutoGoOpMode::Learn
            && let Ok(Some(tc)) = self.timecode_reader.read_timecode()
        {
            cue.autogo_timecode = Some(tc);
        }
    }

    fn max_time_until_go(&self, cue: &Cue) -> f64 {
        10.0
    }
}

impl AutoTimecode {
    pub fn new() -> Self {
        Self {
            mode: AutoGoOpMode::Off,
            offset: TimecodeOffset::new(Timecode::default(), false),
            timecode_reader: TimecodeCollector::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoFollow {
    #[serde(skip)]
    app_ctx: Context,
    #[serde(skip)]
    last_go_time: f64,
    mode: AutoGoOpMode,
}

impl AutoFollow {
    pub fn new(app_ctx: Context) -> Self {
        Self {
            app_ctx,
            last_go_time: 0.0,
            mode: AutoGoOpMode::Off,
        }
    }

    pub fn elapsed(&self) -> f64 {
        let now = self.app_ctx.input(|i| i.time);
        now - self.last_go_time
    }
}

impl AutoGo for AutoFollow {
    fn time_until_go(&self, cue: &Cue) -> f64 {
        if (self.mode == AutoGoOpMode::Ctrl || self.mode == AutoGoOpMode::Hint)
            && let Some(ms) = cue.autogo_delay_ms
        {
            let s = ms as f64 / 1000.0;
            return s - self.elapsed();
        }
        f64::INFINITY
    }

    fn max_time_until_go(&self, cue: &Cue) -> f64 {
        if let Some(ms) = cue.autogo_delay_ms {
            return ms as f64 / 1000.0;
        }
        f64::INFINITY
    }

    fn mode_mut(&mut self) -> &mut AutoGoOpMode {
        &mut self.mode
    }

    fn go_happened(&mut self, cue: &mut Cue) {
        if self.mode == AutoGoOpMode::Learn {
            cue.autogo_delay_ms = Some((self.elapsed() * 1000.0) as u16)
        }
        self.last_go_time = self.app_ctx.input(|i| i.time);
    }
}
