use egui::Widget;
use ks_common_generic::str::StaticString;
use ks_common_ui::{
    components,
    traits::{
        ConfigurationWidget, InlineWidget, InlineWidgetMenu, SubstitutedAutoInlineWidgetMenu,
    },
};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

#[derive(Clone)]
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

#[derive(Clone)]
pub struct ErrorLog {
    errors: Vec<LoggedError>,
}

impl ErrorLog {
    pub fn new() -> Self {
        Self { errors: vec![] }
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

    pub fn update(&mut self, ctx: &egui::Context, time: f64) {
        let errstr: Option<String> =
            ctx.data_mut(|w| w.get_temp_mut_or("tekst.error.msg".into(), None).take());

        if let Some(err) = errstr {
            self.log(time, err);
        }
    }

    pub fn primary_error(&self) -> Option<String> {
        Some(self.errors.first()?.message.clone())
    }

    pub fn num_errors(&self) -> usize {
        self.errors.len()
    }
}

impl InlineWidget for ErrorLog {
    fn draw(&mut self, ui: &mut egui::Ui, label: &str) -> egui::Response {
        components::TextDisplay::new(
            &match self.primary_error() {
                Some(e) => format!("(1/{}) {}", self.num_errors(), e),
                None => "(0/0)".to_string(),
            },
            48,
        )
        .label(label)
        .color_o(
            self.num_errors()
                .gt(&0)
                .then_some(ui.visuals().error_fg_color),
        )
        .ui(ui)
    }
}

impl ConfigurationWidget for ErrorLog {
    fn grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Alerts & Errors");
            let mut queue_delete = None;
            for (i, err) in self.errors.iter().enumerate() {
                if StaticString::<64>::new(&err.message)
                    .inline_widget(ui, "(click to dismiss)")
                    .clicked()
                {
                    queue_delete = Some(i);
                }
                ui.end_row();
            }
            if let Some(i) = queue_delete {
                self.errors.remove(i);
            }
        });
    }
}

pub fn log_error_msg(ctx: &egui::Context, e: &impl ToString) {
    ctx.data_mut(|w| *w.get_temp_mut_or("tekst.error.msg".into(), None) = Some(e.to_string()));
}

pub fn log_if_error<T, E>(ui: &egui::Ui, r: Result<T, E>) -> Option<T>
where
    E: ToString,
{
    match r {
        Ok(v) => Some(v),
        Err(e) => {
            log_error_msg(ui.ctx(), &e);
            None
        }
    }
}
