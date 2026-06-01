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
            Some(err) => Self::COUNTDOWN_LENGTH - (time - err.time),
            None => 0.0,
        }
    }

    pub fn countdown(&self) -> f64 {
        self.primary_error_countdown
    }

    pub fn primary_error(&self) -> Option<String> {
        Some(self.errors.first()?.message.clone())
    }

    pub fn num_errors(&self) -> usize {
        self.errors.len()
    }
}
