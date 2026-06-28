use egui::pos2;

use crate::DISPLAY_NUM_LINES;

use crate::app::TekstApp;
use crate::autogo;
use crate::elements::color_with_default;
use crate::elements::property_with_default;
use crate::elements::text_lines;
use egui::Align;

pub(crate) struct ScriptLineListDelegate<'a> {
    pub(crate) app: &'a mut TekstApp,
}

impl<'a> ScriptLineListDelegate<'a> {
    pub(crate) fn new(app: &'a mut TekstApp) -> Self {
        Self { app }
    }
}

const EXTRA_ROWS_ABOVE: u64 = 8;
const EXTRA_ROWS_BELOW: u64 = 8;

impl egui_table::TableDelegate for ScriptLineListDelegate<'_> {
    fn row_ui(&mut self, ui: &mut egui::Ui, row_nr: u64) {
        if row_nr.is_multiple_of(2) {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
    }

    fn default_row_height(&self) -> f32 {
        16.0 + 16.0 * DISPLAY_NUM_LINES as f32
    }

    fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::HeaderCellInfo) {
        match cell.col_range.start {
            0 => ui.label("Line"),
            2 => ui.label("Brightness / Fade"),
            4 => ui.label("Design"),
            7 => ui.label("Auto Follow"),
            _ => ui.label("N/A"),
        };
    }

    fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
        let col_nr = cell.col_nr;
        let row_nr = cell.row_nr;

        let rect = ui.max_rect();
        let x = rect.right();
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter()
            .line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
        ui.painter()
            .rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);

        let global_style = self.app.global_style;

        let mut interaction_happened = false;
        let autofollow_progress = self.app.autogo.progress(&self.app.selected_cue());

        if let Some(seq) = self.app.selected_sequence() {
            if row_nr < EXTRA_ROWS_ABOVE {
                return;
            }
            let cue_nr = row_nr.saturating_sub(EXTRA_ROWS_ABOVE);
            let this_is_selected = cue_nr as usize == seq.sequence.cue_pointer;

            if cue_nr >= seq.sequence.cues.len() as u64 {
                return;
            }

            if this_is_selected {
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    ui.visuals().widgets.noninteractive.fg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let cue = &seq.sequence.cues[cue_nr as usize];
            ui.centered_and_justified(|ui| {
                match col_nr {
                    0 => {
                        if ui.monospace(cue.ident.clone()).clicked() {
                            seq.sequence.cue_pointer = cue_nr as usize;
                            interaction_happened = true;
                        }
                    }
                    1 => {
                        text_lines(cue, ui, false, global_style);
                    }
                    2 => {
                        property_with_default(ui, cue.brightness, &global_style.brightness);
                    }
                    3 => {
                        property_with_default(ui, cue.fade_speed, &global_style.fade_speed);
                    }
                    4 => {
                        color_with_default(ui, cue.text_color, &global_style.text_color);
                    }
                    5 => {
                        property_with_default(ui, cue.text_align, &global_style.text_align);
                    }
                    6 => {
                        property_with_default(ui, cue.text_font, &global_style.text_font);
                    }
                    7 => {
                        if let Some(val) = cue.autogo_delay_ms {
                            ui.monospace(format!(
                                "{:.3} s",
                                if this_is_selected {
                                    1.0 - autofollow_progress
                                } else {
                                    1.0
                                } * val as f32
                                    / 1000.0
                            ));
                        };
                    }
                    8 => {
                        if let Some(val) = cue.autogo_timecode {
                            ui.monospace(val.to_string());
                        };
                    }
                    _ => {
                        ui.label("Unknown field");
                    }
                };
            });
            if (cue_nr as usize) < seq.sequence.cue_pointer {
                ui.painter()
                    .rect_filled(rect, 0.0, ui.visuals().panel_fill.gamma_multiply(0.5));
            }
            if interaction_happened {
                self.app.reset_follow_time();
            }
        }
    }
}

pub fn cue_table(app: &mut TekstApp, ui: &mut egui::Ui) {
    let mut table = egui_table::Table::new()
        .headers([egui_table::HeaderRow {
            height: 32.0,
            groups: vec![(0..2), (2..4), (4..7), (7..9)],
        }])
        .num_sticky_cols(2)
        .columns([
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(256.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(128.0).resizable(false),
            egui_table::Column::new(128.0).resizable(false),
        ]);
    let autoscroll = app.autoscroll;
    if let Some(seq) = app.selected_sequence() {
        if autoscroll {
            table = table.scroll_to_row(
                (seq.sequence.cue_pointer as u64).saturating_add(EXTRA_ROWS_ABOVE),
                Some(Align::Center),
            );
        }
        table =
            table.num_rows(seq.sequence.cues.len() as u64 + EXTRA_ROWS_ABOVE + EXTRA_ROWS_BELOW);
    } else {
        table = table.num_rows(1);
    }
    if table
        .show(ui, &mut ScriptLineListDelegate::new(app))
        .dragged()
    {
        app.autoscroll = false;
    };
}
