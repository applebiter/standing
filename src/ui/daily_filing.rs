//! The Daily Filing: record status events as they occur.

use std::collections::HashMap;

use egui::{Align, Layout, RichText, Sense, Ui};
use egui_extras::{Column, TableBuilder};
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};

use super::Notice;
use super::chart::movement_color;
use crate::app::Session;
use crate::db;
use crate::engine::{index, market};
use crate::model::comparable::Comparable;
use crate::model::event::{Event, EventKind};
use crate::model::optional_bounded;
use crate::theme::{self, Palette, space};
use crate::{copy, format};

const FORM_WIDTH: f32 = 360.0;
/// Filings older than this are not listed.
const LISTED_DAYS: i64 = 30;

struct Loaded {
    revision: u64,
    events: Vec<Event>,
    comparables: Vec<Comparable>,
    /// Every Comparable, including former ones, for naming past filings.
    tickers: HashMap<i64, String>,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    kind: Option<EventKind>,
    comparable: Option<i64>,
    note: String,
    attempted: bool,
    notice: Option<Notice>,
    confirm_withdraw: Option<Event>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::filing::TITLE, copy::filing::SUBTITLE);
    if state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision)
    {
        match load(session) {
            Ok(l) => state.loaded = Some(l),
            Err(e) => {
                state.notice = Some(Notice::Error(format!(
                    "{} {e:#}",
                    copy::filing::LOAD_FAILED
                )))
            }
        }
    }
    super::notice(ui, p, &mut state.notice);
    if state.loaded.is_none() {
        return;
    }

    let width = ui.available_width();
    if width >= FORM_WIDTH + 420.0 {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            column(ui, FORM_WIDTH, |ui| form(ui, p, session, state));
            column(ui, width - FORM_WIDTH - space::MD, |ui| {
                ledger(ui, p, session, state)
            });
        });
    } else {
        form(ui, p, session, state);
        ui.add_space(space::MD);
        ledger(ui, p, session, state);
    }
    confirm_withdraw(ui, p, session, state);
}

fn column(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(egui::vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_width(width);
        add(ui);
    });
}

fn load(session: &Session) -> db::Result<Loaded> {
    let all = db::comparables::all(&session.conn)?;
    Ok(Loaded {
        revision: session.revision,
        events: db::events::all(&session.conn)?,
        tickers: all.iter().map(|c| (c.id, c.ticker.clone())).collect(),
        comparables: all.into_iter().filter(|c| c.active).collect(),
    })
}

fn form(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let has_comparables = state
        .loaded
        .as_ref()
        .is_some_and(|l| !l.comparables.is_empty());
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        super::overline(ui, p, copy::filing::FILE_AN_EVENT);
        ui.add_space(space::SM);
        for (heading, kinds) in [
            (
                copy::filing::GROUP_OWN,
                EventKind::ALL
                    .iter()
                    .filter(|k| !k.concerns_comparable())
                    .collect::<Vec<_>>(),
            ),
            (
                copy::filing::GROUP_OBSERVED,
                EventKind::ALL
                    .iter()
                    .filter(|k| k.concerns_comparable())
                    .collect(),
            ),
        ] {
            ui.add_space(space::XS);
            ui.label(RichText::new(heading).small().color(p.text_secondary));
            for &kind in kinds {
                let enabled = !kind.concerns_comparable() || has_comparables;
                if kind_row(ui, p, kind, state.kind == Some(kind), enabled) {
                    state.kind = Some(kind);
                    state.attempted = false;
                }
            }
        }
        if !has_comparables {
            ui.label(
                RichText::new(copy::filing::OBSERVED_NEEDS_COMPARABLES)
                    .small()
                    .color(p.text_secondary),
            );
        }

        let Some(kind) = state.kind else {
            return;
        };
        ui.add_space(space::MD);
        ui.label(
            RichText::new(copy::filing::description(kind))
                .small()
                .color(p.text_secondary),
        );
        ui.add_space(space::SM);

        if kind.concerns_comparable() {
            let loaded = state.loaded.as_ref().expect("loaded");
            let selected = state
                .comparable
                .and_then(|id| loaded.comparables.iter().find(|c| c.id == id));
            ui.label(RichText::new(copy::filing::COMPARABLE).family(theme::family::sans_medium()));
            egui::ComboBox::from_id_salt("filing-comparable")
                .width(ui.available_width())
                .selected_text(
                    selected.map_or(copy::filing::CHOOSE_COMPARABLE.to_owned(), |c| {
                        format!("{}  {}", c.ticker, c.name)
                    }),
                )
                .show_ui(ui, |ui| {
                    for c in &loaded.comparables {
                        ui.selectable_value(
                            &mut state.comparable,
                            Some(c.id),
                            format!("{}  {}", c.ticker, c.name),
                        );
                    }
                });
            if state.attempted && selected.is_none() {
                ui.label(
                    RichText::new(copy::filing::COMPARABLE_REQUIRED)
                        .small()
                        .color(p.danger),
                );
            }
            ui.add_space(space::SM);
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new(copy::filing::NOTE).family(theme::family::sans_medium()));
            ui.label(
                RichText::new(copy::form::OPTIONAL)
                    .small()
                    .color(p.text_secondary),
            );
        });
        ui.add(
            egui::TextEdit::singleline(&mut state.note)
                .hint_text(copy::filing::NOTE_HINT)
                .desired_width(f32::INFINITY),
        );
        let note = optional_bounded(&state.note);
        if let Err(e) = &note {
            ui.label(RichText::new(copy::form::error(*e)).small().color(p.danger));
        }
        ui.add_space(space::MD);

        if super::primary_button(ui, p, copy::filing::SUBMIT).clicked() {
            state.attempted = true;
            let comparable = if kind.concerns_comparable() {
                state.comparable.filter(|id| {
                    state
                        .loaded
                        .as_ref()
                        .is_some_and(|l| l.comparables.iter().any(|c| c.id == *id))
                })
            } else {
                None
            };
            if let (Ok(note), true) = (note, !kind.concerns_comparable() || comparable.is_some()) {
                state.notice = Some(
                    match db::events::insert(
                        &session.conn,
                        kind,
                        comparable,
                        note.as_deref(),
                        Timestamp::now(),
                    ) {
                        Ok(_) => {
                            session.revision += 1;
                            state.note.clear();
                            state.attempted = false;
                            Notice::Info(copy::filing::filed(kind))
                        }
                        Err(e) => Notice::Error(format!("{} {e:#}", copy::filing::WRITE_FAILED)),
                    },
                );
            }
        }
    });
}

/// One selectable event type, with its weight. Returns whether it was chosen.
fn kind_row(ui: &mut Ui, p: &Palette, kind: EventKind, selected: bool, enabled: bool) -> bool {
    let height = 30.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if selected {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.5),
        );
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(2.0, height));
        ui.painter().rect_filled(bar, 0.0, p.accent);
    } else if enabled && response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.25),
        );
    }
    let text = if !enabled {
        p.text_secondary.gamma_multiply(0.6)
    } else if selected {
        p.text_primary
    } else {
        p.text_primary.gamma_multiply(0.85)
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(space::SM + 2.0, 0.0),
        egui::Align2::LEFT_CENTER,
        copy::filing::label(kind),
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
        text,
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(space::SM, 0.0),
        egui::Align2::RIGHT_CENTER,
        format::signed(kind.weight()),
        egui::FontId::new(12.5, theme::family::mono_medium()),
        if enabled {
            p.text_secondary
        } else {
            p.text_secondary.gamma_multiply(0.6)
        },
    );
    enabled
        && response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
}

fn ledger(ui: &mut Ui, p: &Palette, session: &Session, state: &mut State) {
    let Some(loaded) = &state.loaded else {
        return;
    };
    let now = Timestamp::now();
    let tz = TimeZone::system();
    let today = market::day_of(now, &tz);

    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                super::overline(ui, p, copy::filing::MOMENTUM);
                let momentum = session.reading.as_ref().map_or(0.0, |r| r.momentum);
                ui.label(
                    RichText::new(format::signed(momentum))
                        .family(theme::family::mono_medium())
                        .size(28.0)
                        .color(movement_color(p, momentum)),
                );
            });
            ui.add_space(space::XL);
            ui.vertical(|ui| {
                super::overline(ui, p, copy::filing::FILED_TODAY);
                let n = loaded
                    .events
                    .iter()
                    .filter(|e| market::day_of(e.occurred_at, &tz) == today)
                    .count();
                ui.label(
                    RichText::new(format::thousands(n as u64))
                        .family(theme::family::mono_medium())
                        .size(28.0),
                );
            });
        });
        ui.label(
            RichText::new(copy::filing::MOMENTUM_NOTE)
                .small()
                .color(p.text_secondary),
        );
    });
    ui.add_space(space::MD);

    let cutoff = market::start_of(today.saturating_sub((LISTED_DAYS - 1).days()), &tz);
    let listed: Vec<&Event> = loaded
        .events
        .iter()
        .rev()
        .filter(|e| e.occurred_at >= cutoff)
        .collect();
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        super::overline(ui, p, copy::filing::RECENT);
        ui.add_space(space::SM);
        if listed.is_empty() {
            ui.label(RichText::new(copy::filing::NO_FILINGS).color(p.text_secondary));
            return;
        }
        let mut withdraw = None;
        TableBuilder::new(ui)
            .id_salt("filings")
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink([false, true])
            .vscroll(false)
            .column(Column::exact(84.0))
            .column(Column::remainder().at_least(100.0).clip(true))
            .column(Column::exact(56.0))
            .column(Column::exact(56.0))
            .column(Column::exact(24.0))
            .header(24.0, |mut header| {
                for h in [
                    copy::filing::COL_FILED,
                    copy::filing::COL_EVENT,
                    copy::filing::COL_WEIGHT,
                    copy::filing::COL_CURRENT,
                    "",
                ] {
                    header.col(|ui| {
                        ui.label(super::overline_text(p, h));
                    });
                }
            })
            .body(|body| {
                body.rows(44.0, listed.len(), |mut row| {
                    let e = listed[row.index()];
                    let local = e.occurred_at.to_zoned(tz.clone());
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(
                                RichText::new(format::date_short(local.date()))
                                    .small()
                                    .color(p.text_secondary),
                            );
                            ui.label(
                                RichText::new(local.strftime("%H:%M").to_string())
                                    .monospace()
                                    .color(p.text_secondary),
                            );
                        });
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.add(egui::Label::new(copy::filing::label(e.kind)).truncate());
                            let ticker = e.comparable_id.and_then(|id| loaded.tickers.get(&id));
                            let detail = match (ticker, &e.note) {
                                (Some(t), Some(n)) => format!("{t} \u{b7} {n}"),
                                (Some(t), None) => t.clone(),
                                (None, Some(n)) => n.clone(),
                                (None, None) => String::new(),
                            };
                            if !detail.is_empty() {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(detail).small().color(p.text_secondary),
                                    )
                                    .truncate(),
                                );
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format::signed(e.kind.weight())).monospace());
                    });
                    row.col(|ui| {
                        let current = index::contribution(e, now);
                        ui.label(
                            RichText::new(format::signed(current))
                                .monospace()
                                .color(p.text_secondary),
                        );
                    });
                    row.col(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("\u{2715}").color(p.text_secondary),
                                )
                                .frame(false),
                            )
                            .on_hover_text(copy::filing::WITHDRAW)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            withdraw = Some(e.clone());
                        }
                    });
                });
            });
        if withdraw.is_some() {
            state.confirm_withdraw = withdraw;
        }
    });
}

fn confirm_withdraw(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let Some(e) = &state.confirm_withdraw else {
        return;
    };
    let decision = super::confirm_dialog(
        ui,
        p,
        "confirm-withdraw-filing",
        copy::filing::WITHDRAW_TITLE,
        &copy::filing::withdraw_body(e.kind),
        copy::filing::WITHDRAW,
        copy::form::CANCEL,
    );
    match decision {
        Some(true) => {
            state.notice = Some(match db::events::delete(&session.conn, e.id) {
                Ok(()) => {
                    session.revision += 1;
                    Notice::Info(copy::filing::withdrawn(e.kind))
                }
                Err(err) => Notice::Error(format!("{} {err:#}", copy::filing::WRITE_FAILED)),
            });
            state.confirm_withdraw = None;
        }
        Some(false) => state.confirm_withdraw = None,
        None => {}
    }
}
