use crate::app::TekstApp;

pub fn handle_autogo_follow(app: &mut TekstApp) {
    if let Some(ms) = app.selected_cue().autogo_delay_ms {
        let now = app.ctx.input(|i| i.time);
        let elapsed = now - app.last_go_time.unwrap_or_default();

        if elapsed > ms as f64 / 1000.0 {
            app.go();
        }
    }
}

pub fn get_autogo_progress(app: &mut TekstApp) -> f32 {
    if let Some(ms) = app.selected_cue().autogo_delay_ms {
        let now = app.ctx.input(|i| i.time) as f32;
        let elapsed = now - app.last_go_time.unwrap_or_default() as f32;

        return elapsed / (ms as f32 / 1000.0);
    }
    0.0
}
