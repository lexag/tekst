use egui::pos2;

use crate::DISPLAY_NUM_LINES;

use crate::app::OpMode;
use crate::app::TekstApp;
use crate::elements::color_with_default;
use crate::elements::property_with_default;
use crate::elements::text_lines;
use egui::Align;
use egui::Color32;
use ks_common_ui::style;

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
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(match cell.col_range.start {
                0 => "Line",
                1 => "Mark",
                2 => "Description",
                3 => "Content",
                4 => "Brightness",
                5 => "Transition",
                6 => "Color",
                7 => "Align",
                8 => "Font",
                9 => "AFW",
                10 => "ATC",
                _ => "N/A",
            });
        });
    }

    fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
        let col_nr = cell.col_nr;
        let row_nr = cell.row_nr;

        let rect = ui.max_rect();
        let x = rect.right();
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;

        let global_style = self.app.global_style;

        let mut interaction_happened = false;
        let autofollow_progress = self.app.autogo.progress(&self.app.selected_cue().clone());

        let opmode = self.app.op_mode;
        if let Some(seq) = self.app.selected_sequence_mut() {
            if row_nr < EXTRA_ROWS_ABOVE {
                edge_strokes(ui, rect, x, stroke);
                return;
            }

            let cue_nr = row_nr.saturating_sub(EXTRA_ROWS_ABOVE);
            let this_is_selected = cue_nr as usize == seq.sequence.cue_pointer;

            if cue_nr >= seq.sequence.cues.len() as u64 {
                edge_strokes(ui, rect, x, stroke);
                return;
            }

            let cue = seq
                .sequence
                .cues
                .get_mut(cue_nr as usize)
                .expect("extra rows are handled");

            if cue.autogo_delay_ms.is_some() || cue.autogo_timecode.is_some() {
                ui.painter()
                    .rect_filled(rect, 0.0, style::ACTIVE_COLOR.gamma_multiply(0.2));
            }
            if cue.mark.is_some() {
                ui.painter()
                    .rect_filled(rect, 0.0, style::ACCENT_COLOR.gamma_multiply(0.2));
            }
            edge_strokes(ui, rect, x, stroke);

            if this_is_selected {
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    ui.visuals().widgets.noninteractive.fg_stroke,
                    egui::StrokeKind::Inside,
                );
            }

            ui.centered_and_justified(|ui| {
                match col_nr {
                    0 => {
                        if opmode == OpMode::Edit {
                            frameless_text(
                                ui,
                                opmode == OpMode::Edit,
                                Align::Center,
                                &mut cue.ident,
                            );
                        } else if ui
                            .monospace(egui::RichText::new(cue.ident.clone()).color(
                                if cue.mark.is_some() {
                                    style::ACCENT_COLOR
                                } else {
                                    Color32::PLACEHOLDER
                                },
                            ))
                            .clicked()
                        {
                            seq.sequence.cue_pointer = cue_nr as usize;
                            interaction_happened = true;
                        }
                    }
                    1 => {
                        let mut text = cue.mark.clone().unwrap_or_default();
                        frameless_text(ui, opmode == OpMode::Edit, Align::Center, &mut text);
                        if text.is_empty() {
                            cue.mark = None;
                        } else {
                            cue.mark = Some(text);
                        }
                    }
                    2 => {
                        frameless_text(
                            ui,
                            opmode == OpMode::Edit,
                            Align::Center,
                            &mut cue.description,
                        );
                    }
                    3 => {
                        text_lines(cue, ui, opmode == OpMode::Edit, global_style);
                    }
                    4 => {
                        property_with_default(ui, cue.brightness, &global_style.brightness);
                    }
                    5 => {
                        property_with_default(ui, cue.fade_speed, &global_style.fade_speed);
                    }
                    6 => {
                        color_with_default(ui, cue.text_color, &global_style.text_color);
                    }
                    7 => {
                        property_with_default(ui, cue.text_align, &global_style.text_align);
                    }
                    8 => {
                        property_with_default(ui, cue.text_font, &global_style.text_font);
                    }
                    9 => {
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
                    10 => {
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

fn edge_strokes(ui: &mut egui::Ui, rect: egui::Rect, x: f32, stroke: egui::Stroke) {
    ui.painter()
        .line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
    ui.painter()
        .rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);
}

fn frameless_text(ui: &mut egui::Ui, interactive: bool, align: Align, text: &mut String) {
    ui.horizontal_centered(|ui| {
        egui::TextEdit::multiline(text)
            .interactive(interactive)
            .vertical_align(align)
            .desired_width(ui.available_width())
            .frame(false)
            .desired_rows(1)
            .show(ui);
    });
}

pub fn cue_table(app: &mut TekstApp, ui: &mut egui::Ui) {
    let mut table = egui_table::Table::new()
        .headers([egui_table::HeaderRow {
            height: 32.0,
            groups: vec![],
        }])
        .num_sticky_cols(1)
        .columns([
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(64.0).resizable(false),
            egui_table::Column::new(256.0).resizable(false),
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
    if let Some(seq) = app.selected_sequence_mut() {
        if autoscroll {
            table = table.scroll_to_row(
                (seq.sequence.cue_pointer as u64).saturating_add(EXTRA_ROWS_ABOVE),
                Some(Align::Center),
            );
            table = table.scroll_to_column(0, Some(Align::Center));
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
    }
}
