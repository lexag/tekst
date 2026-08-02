use egui::Widget;
use ks_common_ui::{
    components,
    traits::InlineWidgetMenu,
};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

struct LoggedError {
    time: f64,
    message: String,
    hash: u64,
}

impl LoggedError {
    pub fn new(time: f64, message: String) -> Self {
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        let hash = hasher.finish();
        Self {
            time,
            message,
            hash,
        }
    }
}

impl Display for LoggedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

pub struct ErrorLog {
    errors: Vec<LoggedError>,
    primary_error_countdown: f64,
}

impl ErrorLog {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            primary_error_countdown: 0.0,
        }
    }

    const COUNTDOWN_LENGTH: f64 = 10.0;

    pub fn log(&mut self, time: f64, message: String) {
        let new_error = LoggedError::new(time, message);
        for error in &mut self.errors {
            if error.hash == new_error.hash {
                error.time = time;
                return;
            }
        }
        self.errors.push(new_error);
    }

    pub fn update(&mut self, time: f64) {
        self.primary_error_countdown = match self.errors.first() {
            Some(err) => {
                let countdown = Self::COUNTDOWN_LENGTH - (time - err.time);
                if countdown < 0.0 {
                    self.errors.remove(0);
                    0.0
                } else {
                    countdown
                }
            }
            None => 0.0,
        }
    }

    pub fn countdown(&self) -> f64 {
        self.primary_error_countdown
    }

    pub fn countdown_progress(&self) -> f64 {
        self.primary_error_countdown / Self::COUNTDOWN_LENGTH
    }

    pub fn primary_error(&self) -> Option<String> {
        Some(self.errors.first()?.message.clone())
    }

    pub fn num_errors(&self) -> usize {
        self.errors.len()
    }
}

impl InlineWidgetMenu for ErrorLog {
    fn inline_widget_menu(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        _add_contents: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        components::TextDisplay::fullwide(&match self.primary_error() {
            Some(e) => format!("(1/{}) {}", self.num_errors(), e),
            None => "(0/0)".to_string(),
        })
        .label(label)
        .color_o(
            self.num_errors()
                .gt(&0)
                .then_some(ui.visuals().error_fg_color),
        )
        .ui(ui)
    }
}

pub fn log_error_msg(ui: &egui::Ui, e: &impl ToString) {
    ui.ctx()
        .data_mut(|w| *w.get_temp_mut_or("tekst.error.msg".into(), String::new()) = e.to_string());
}

pub fn log_if_error<T, E>(ui: &egui::Ui, r: Result<T, E>) -> Option<T>
where
    E: ToString,
{
    match r {
        Ok(v) => Some(v),
        Err(e) => {
            log_error_msg(ui, &e);
            None
        }
    }
}
