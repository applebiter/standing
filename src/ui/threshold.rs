//! The Enough Threshold, and its history.

use egui::{Align, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use jiff::Timestamp;

use super::Notice;
use crate::app::Session;
use crate::db::{self, threshold::Entry, threshold::Reason};
use crate::model::{self, FieldError};
use crate::theme::{self, Palette, space};
use crate::{copy, format};

const CARD_WIDTH: f32 = 820.0;

struct Loaded {
    revision: u64,
    history: Vec<Entry>,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    input: String,
    error: Option<FieldError>,
    notice: Option<Notice>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::threshold::TITLE, copy::threshold::SUBTITLE);
    if state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision)
    {
        match db::threshold::history(&session.conn) {
            Ok(history) => {
                state.loaded = Some(Loaded {
                    revision: session.revision,
                    history,
                })
            }
            Err(e) => {
                state.notice = Some(Notice::Error(format!(
                    "{} {e:#}",
                    copy::threshold::LOAD_FAILED
                )))
            }
        }
    }
    super::notice(ui, p, &mut state.notice);
    let Some(loaded) = &state.loaded else {
        return;
    };
    let current = loaded.history.first().cloned();
    let width = ui.available_width().min(CARD_WIDTH);

    theme::card(p).show(ui, |ui| {
        ui.set_width(width - 2.0 * space::LG - 2.0);
        match &current {
            Some(entry) => {
                super::overline(ui, p, copy::threshold::CURRENT);
                ui.label(
                    RichText::new(format::index(entry.new))
                        .family(theme::family::display())
                        .size(56.0),
                );
                ui.label(
                    RichText::new(copy::threshold::set_on(
                        &format::local_date(&entry.changed_at)
                            .map(format::date_long)
                            .unwrap_or_default(),
                    ))
                    .small()
                    .color(p.text_secondary),
                );
                if let Some(reading) = &session.reading {
                    ui.add_space(space::MD);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::XL;
                        for (label, value) in [
                            (copy::threshold::PSI, format::index(reading.psi)),
                            (
                                copy::threshold::SHORTFALL,
                                format::index((entry.new - reading.psi).max(0.0)),
                            ),
                        ] {
                            ui.vertical(|ui| {
                                ui.label(super::overline_text(p, label));
                                ui.label(RichText::new(value).monospace());
                            });
                        }
                    });
                }
            }
            None => {
                ui.label(
                    RichText::new(copy::threshold::NOT_SET)
                        .family(theme::family::serif())
                        .size(22.0),
                );
                ui.add_space(space::XS);
                ui.label(RichText::new(copy::threshold::NOT_SET_BODY).color(p.text_secondary));
            }
        }

        ui.add_space(space::LG);
        super::overline(
            ui,
            p,
            if current.is_some() {
                copy::threshold::REVISE
            } else {
                copy::threshold::SET
            },
        );
        ui.add_space(space::XS);
        let mut submitted = false;
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.input)
                    .hint_text(
                        current
                            .as_ref()
                            .map_or(String::new(), |e| format::index(e.new)),
                    )
                    .desired_width(160.0)
                    .font(egui::TextStyle::Monospace),
            );
            if response.changed() {
                state.error = None;
            }
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if super::primary_button(ui, p, copy::threshold::SAVE).clicked() || enter {
                submitted = true;
            }
        });
        match state.error {
            Some(e) => {
                ui.label(RichText::new(copy::form::error(e)).small().color(p.danger));
            }
            None => {
                ui.label(
                    RichText::new(copy::threshold::FIELD_HELP)
                        .small()
                        .color(p.text_secondary),
                );
            }
        }
        if submitted {
            submit(session, state, current.as_ref());
        }
    });

    ui.add_space(space::MD);
    let Some(loaded) = &state.loaded else {
        return;
    };
    theme::card(p).show(ui, |ui| {
        ui.set_width(width - 2.0 * space::LG - 2.0);
        super::overline(ui, p, copy::threshold::HISTORY);
        ui.add_space(space::SM);
        if loaded.history.is_empty() {
            ui.label(RichText::new(copy::threshold::NO_HISTORY).color(p.text_secondary));
            return;
        }
        TableBuilder::new(ui)
            .id_salt("threshold-history")
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink([false, true])
            .vscroll(false)
            .column(Column::exact(110.0))
            .column(Column::exact(80.0))
            .column(Column::exact(80.0))
            .column(Column::remainder().at_least(160.0).clip(true))
            .header(24.0, |mut header| {
                for h in [
                    copy::threshold::COL_DATE,
                    copy::threshold::COL_PRIOR,
                    copy::threshold::COL_NEW,
                    copy::threshold::COL_REASON,
                ] {
                    header.col(|ui| {
                        ui.label(super::overline_text(p, h));
                    });
                }
            })
            .body(|body| {
                body.rows(32.0, loaded.history.len(), |mut row| {
                    let e = &loaded.history[row.index()];
                    row.col(|ui| {
                        ui.label(
                            RichText::new(
                                format::local_date(&e.changed_at)
                                    .map(format::date_short)
                                    .unwrap_or_default(),
                            )
                            .color(p.text_secondary),
                        );
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(
                                e.prior
                                    .map(format::index)
                                    .unwrap_or_else(|| copy::comparables::NO_VALUE.to_owned()),
                            )
                            .monospace()
                            .color(p.text_secondary),
                        );
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format::index(e.new)).monospace());
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(copy::threshold::reason(e.reason)).truncate());
                    });
                });
            });
    });
}

fn submit(session: &mut Session, state: &mut State, current: Option<&Entry>) {
    let value = match model::threshold(&state.input) {
        Ok(v) => v,
        Err(e) => {
            state.error = Some(e);
            return;
        }
    };
    if current.is_some_and(|c| c.new == value) {
        state.notice = Some(Notice::Info(copy::threshold::UNCHANGED.to_owned()));
        return;
    }
    let reason = if current.is_some() {
        Reason::User
    } else {
        Reason::Initial
    };
    state.notice = Some(
        match db::threshold::set(&session.conn, value, reason, Timestamp::now()) {
            Ok(()) => {
                state.input.clear();
                // Recomputation follows, and applies the threshold at once.
                session.revision += 1;
                Notice::Info(copy::threshold::SAVED.to_owned())
            }
            Err(e) => Notice::Error(format!("{} {e:#}", copy::threshold::WRITE_FAILED)),
        },
    );
}
