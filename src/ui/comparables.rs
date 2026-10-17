//! Comparables: the peer set, with coverage of ratings for each.

use std::collections::{HashMap, HashSet};

use egui::{Align, Layout, RichText, Sense, Ui};
use egui_extras::{Column, TableBuilder};

use super::Notice;
use super::comparable_form::{self, FormState, Outcome};
use crate::app::Session;
use crate::db::{self, comparables};
use crate::engine::index::Reading;
use crate::model::comparable::Comparable;
use crate::model::rating::Subject;
use crate::theme::{self, Palette, space};
use crate::{copy, format};

/// Rating coverage for one Comparable, over applicable dimensions.
#[derive(Default)]
struct Coverage {
    rated: usize,
    last_rated: Option<String>,
}

struct Loaded {
    revision: u64,
    comparables: Vec<Comparable>,
    coverage: HashMap<i64, Coverage>,
    applicable: usize,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    form: Option<FormState>,
    notice: Option<Notice>,
    confirm_delete: Option<Comparable>,
    show_inactive: bool,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    if let Some(form) = &mut state.form {
        match comparable_form::show(ui, p, session, form) {
            Outcome::Open => {}
            Outcome::Cancelled => state.form = None,
            Outcome::Saved(text) => {
                state.notice = Some(Notice::Info(text));
                state.form = None;
            }
            Outcome::DeleteRequested => state.confirm_delete = form.editing().cloned(),
        }
        confirm_delete(ui, p, session, state);
        return;
    }

    super::page(ui, "comparables", |ui| list(ui, p, session, state));
}

fn list(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(copy::comparables::TITLE).text_style(egui::TextStyle::Heading));
            ui.add_space(space::XS);
            ui.label(RichText::new(copy::comparables::SUBTITLE).color(p.text_secondary));
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if super::primary_button(ui, p, copy::comparables::ADD).clicked() {
                state.form = Some(FormState::new());
            }
        });
    });
    ui.add_space(space::XL);

    if state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision)
    {
        match load(session) {
            Ok(loaded) => state.loaded = Some(loaded),
            Err(e) => {
                state.notice = Some(Notice::Error(format!(
                    "{} {e:#}",
                    copy::comparables::LOAD_FAILED
                )));
            }
        }
    }
    super::notice(ui, p, &mut state.notice);
    let Some(loaded) = &state.loaded else {
        return;
    };

    if loaded.comparables.is_empty() {
        ui.add_space(space::XXL);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(copy::comparables::EMPTY_TITLE)
                    .family(theme::family::serif())
                    .size(22.0),
            );
            ui.add_space(space::SM);
            ui.label(RichText::new(copy::comparables::EMPTY_BODY).color(p.text_secondary));
        });
        return;
    }

    if loaded.comparables.iter().any(|c| !c.active) {
        ui.checkbox(&mut state.show_inactive, copy::comparables::SHOW_INACTIVE);
        ui.add_space(space::SM);
    }
    let shown: Vec<&Comparable> = loaded
        .comparables
        .iter()
        .filter(|c| c.active || state.show_inactive)
        .collect();
    if let Some(c) = table(ui, p, loaded, session.reading.as_ref(), &shown) {
        state.form = Some(FormState::edit(c));
    }
}

fn load(session: &Session) -> db::Result<Loaded> {
    let comparables = comparables::all(&session.conn)?;
    let applicable: HashSet<String> = db::dimensions::visible(&session.conn)?
        .into_iter()
        .filter(|d| d.is_active())
        .map(|d| d.id)
        .collect();
    let mut coverage: HashMap<i64, Coverage> = HashMap::new();
    for ((subject, dim), rating) in db::ratings::current(&session.conn)? {
        let Subject::Comparable(id) = subject else {
            continue;
        };
        if !applicable.contains(&dim) {
            continue;
        }
        let c = coverage.entry(id).or_default();
        c.rated += 1;
        if c.last_rated.as_ref().is_none_or(|t| *t < rating.rated_at) {
            c.last_rated = Some(rating.rated_at);
        }
    }
    Ok(Loaded {
        revision: session.revision,
        comparables,
        coverage,
        applicable: applicable.len(),
    })
}

/// Draws the table and returns the Comparable whose row was clicked, if any.
fn table(
    ui: &mut Ui,
    p: &Palette,
    loaded: &Loaded,
    reading: Option<&Reading>,
    shown: &[&Comparable],
) -> Option<Comparable> {
    let wide = ui.available_width() >= 840.0;
    let mut clicked = None;

    let mut builder = TableBuilder::new(ui)
        .striped(true)
        .sense(Sense::click())
        .cell_layout(Layout::left_to_right(Align::Center))
        .auto_shrink([false, true])
        .column(Column::exact(84.0))
        .column(Column::initial(150.0).clip(true))
        .column(Column::remainder().at_least(110.0).clip(true))
        .column(Column::exact(72.0))
        .column(Column::exact(100.0))
        .column(Column::exact(96.0));
    if wide {
        builder = builder
            .column(Column::exact(104.0))
            .column(Column::exact(96.0));
    }

    builder
        .header(28.0, |mut header| {
            let mut head = |text: &str, hint: Option<&str>| {
                header.col(|ui| {
                    let r = ui.label(super::overline_text(p, text));
                    if let Some(hint) = hint {
                        r.on_hover_text(hint);
                    }
                });
            };
            head(copy::comparables::COL_TICKER, None);
            head(copy::comparables::COL_NAME, None);
            head(copy::comparables::COL_RELATIONSHIP, None);
            head(copy::comparables::COL_INDEX, None);
            head(
                copy::comparables::COL_SPREAD,
                Some(copy::comparables::SPREAD_HINT),
            );
            head(copy::comparables::COL_COVERAGE, None);
            if wide {
                head(copy::comparables::COL_LAST_RATED, None);
                head(copy::comparables::COL_STATUS, None);
            }
        })
        .body(|body| {
            body.rows(38.0, shown.len(), |mut row| {
                let c = shown[row.index()];
                let coverage = loaded.coverage.get(&c.id);
                let primary = if c.active {
                    p.text_primary
                } else {
                    p.text_secondary
                };
                let cell = |row: &mut egui_extras::TableRow<'_, '_>, text: RichText| {
                    row.col(|ui| {
                        ui.add(egui::Label::new(text).truncate().selectable(false));
                    });
                };
                cell(
                    &mut row,
                    RichText::new(&c.ticker)
                        .family(theme::family::mono_medium())
                        .color(primary),
                );
                cell(&mut row, RichText::new(&c.name).color(primary));
                cell(
                    &mut row,
                    RichText::new(c.relationship.as_deref().unwrap_or("")).color(p.text_secondary),
                );
                let figure = |v: Option<String>| {
                    RichText::new(v.unwrap_or_else(|| copy::comparables::NO_VALUE.to_owned()))
                        .monospace()
                        .color(primary)
                };
                let value = reading.and_then(|r| r.comparable(c.id)).map(|r| r.value);
                cell(&mut row, figure(value.map(format::index)));
                let spread = reading.and_then(|r| r.spread(c.id));
                cell(&mut row, figure(spread.map(format::signed)));
                cell(
                    &mut row,
                    RichText::new(copy::comparables::coverage(
                        coverage.map_or(0, |c| c.rated),
                        loaded.applicable,
                    ))
                    .monospace()
                    .color(primary),
                );
                if wide {
                    let last = coverage
                        .and_then(|c| c.last_rated.as_deref())
                        .and_then(format::local_date)
                        .map(format::date_short)
                        .unwrap_or_else(|| copy::comparables::NOT_RATED.to_owned());
                    cell(&mut row, RichText::new(last).color(p.text_secondary));
                    cell(
                        &mut row,
                        RichText::new(if c.active {
                            copy::comparables::STATUS_ACTIVE
                        } else {
                            copy::comparables::STATUS_INACTIVE
                        })
                        .color(p.text_secondary),
                    );
                }
                let response = row.response();
                if response.hovered() {
                    response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    clicked = Some(c.clone());
                }
            });
        });
    clicked
}

fn confirm_delete(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let Some(c) = &state.confirm_delete else {
        return;
    };
    let decision = super::confirm_dialog(
        ui,
        p,
        "confirm-delete-comparable",
        copy::comparables::DELETE_TITLE,
        &copy::comparables::delete_body(&c.name),
        copy::comparables::DELETE_CONFIRM,
        copy::form::CANCEL,
    );
    match decision {
        Some(true) => {
            let name = c.name.clone();
            state.notice = Some(match comparables::delete(&mut session.conn, c.id) {
                Ok(()) => {
                    session.revision += 1;
                    state.form = None;
                    Notice::Info(copy::comparables::deleted(&name))
                }
                Err(e) => Notice::Error(format!("{} {e:#}", copy::comparables::WRITE_FAILED)),
            });
            state.confirm_delete = None;
        }
        Some(false) => state.confirm_delete = None,
        None => {}
    }
}
