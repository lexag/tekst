use crate::{app::TekstApp, cue::Cue, timecode::TimecodeReader};
use egui::Context;
use std::fmt::Display;

pub trait AutoGo {
    fn time_until_go(&self, cue: &Cue) -> f64;
    fn max_time_until_go(&self, cue: &Cue) -> f64;

    fn progress(&self, cue: &Cue) -> f32 {
        1.0 - (self.time_until_go(cue) / self.max_time_until_go(cue)) as f32
    }

    fn requests_go(&self, cue: &Cue) -> bool;

    fn mode_mut(&mut self) -> &mut AutoGoOpMode;

    fn go_happened(&mut self, cue: &mut Cue);
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum AutoGoOpMode {
    Off,
    Ctrl,
    Learn,
}

impl Display for AutoGoOpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoGoOpMode::Off => write!(f, "Off"),
            AutoGoOpMode::Ctrl => write!(f, "Ctrl"),
            AutoGoOpMode::Learn => write!(f, "Learn"),
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

    pub fn requests_go(&self, cue: &Cue) -> bool {
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
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AutoTimecode {
    #[serde(skip)]
    pub timecode_reader: TimecodeReader,
    pub mode: AutoGoOpMode,
}

impl AutoGo for AutoTimecode {
    fn time_until_go(&self, cue: &Cue) -> f64 {
        f64::INFINITY
    }

    fn requests_go(&self, cue: &Cue) -> bool {
        false
    }

    fn mode_mut(&mut self) -> &mut AutoGoOpMode {
        &mut self.mode
    }

    fn go_happened(&mut self, cue: &mut Cue) {}

    fn max_time_until_go(&self, cue: &Cue) -> f64 {
        f64::INFINITY
    }
}

impl AutoTimecode {
    pub fn new() -> Self {
        Self {
            mode: AutoGoOpMode::Off,
            timecode_reader: TimecodeReader::new(),
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
        if self.mode == AutoGoOpMode::Ctrl
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

    fn requests_go(&self, cue: &Cue) -> bool {
        if self.time_until_go(cue) <= 0.0 {
            return true;
        }
        false
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
