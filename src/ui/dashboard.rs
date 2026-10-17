//! The Dashboard: ticker strip, Personal Standing Index, Spotlight Monitor,
//! index history, Positions Held, Comparables and notices.

use std::collections::HashMap;

use egui::{Align, Layout, RichText, Ui};
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};

use super::chart::{self, movement_color};
use super::ticker_strip::{self, Quote};
use crate::app::Session;
use crate::db::{self, notices::Notice};
use crate::engine::market::{self, Candle};
use crate::engine::positions::{self, Position};
use crate::engine::{run, spotlight};
use crate::model::comparable::{Comparable, SELF_TICKER};
use crate::model::rating::Subject;
use crate::theme::{self, Palette, space};
use crate::{copy, format};

const SPOTLIGHT_WIDTH: f32 = 300.0;
const CHART_HEIGHT: f32 = 260.0;
const NOTICES_SHOWN: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Range {
    #[default]
    Month,
    Quarter,
    Year,
    All,
}

impl Range {
    const ALL: [Self; 4] = [Self::Month, Self::Quarter, Self::Year, Self::All];

    fn label(self) -> &'static str {
        match self {
            Self::Month => copy::dashboard::RANGE_MONTH,
            Self::Quarter => copy::dashboard::RANGE_QUARTER,
            Self::Year => copy::dashboard::RANGE_YEAR,
            Self::All => copy::dashboard::RANGE_ALL,
        }
    }

    /// Days shown, counting today; `None` for everything on record.
    fn days(self) -> Option<i64> {
        match self {
            Self::Month => Some(30),
            Self::Quarter => Some(91),
            Self::Year => Some(365),
            Self::All => None,
        }
    }
}

struct Loaded {
    revision: u64,
    range: Range,
    today: jiff::civil::Date,
    candles: Vec<Candle>,
    threshold: Option<f64>,
    /// The value each change is measured from: the previous close, or failing
    /// that today's first value.
    reference: HashMap<Subject, f64>,
    /// Whether the user's reference value is a previous close (else today's open).
    own_reference_is_close: bool,
    comparables: Vec<Comparable>,
    names: HashMap<String, String>,
    positions: Vec<Position>,
    notices: Vec<Notice>,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    range: Range,
    error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let tz = TimeZone::system();
    let now = Timestamp::now();
    let today = market::day_of(now, &tz);
    if state.loaded.as_ref().is_none_or(|l| {
        l.revision != session.revision || l.range != state.range || l.today != today
    }) {
        match load(session, state.range, today, &tz) {
            Ok(l) => {
                state.loaded = Some(l);
                state.error = None;
            }
            Err(e) => state.error = Some(format!("{} {e:#}", copy::dashboard::LOAD_FAILED)),
        }
    }
    if let Some(err) = &state.error {
        ui.label(RichText::new(err).color(p.danger));
        ui.add_space(space::MD);
    }
    let (Some(loaded), Some(reading)) = (&state.loaded, &session.reading) else {
        return;
    };

    let change = |subject: Subject, value: f64| loaded.reference.get(&subject).map(|r| value - r);
    let mut quotes = vec![Quote {
        ticker: SELF_TICKER.to_owned(),
        value: reading.psi,
        change: change(Subject::Own, reading.psi),
    }];
    for c in &loaded.comparables {
        if let Some(r) = reading.comparable(c.id) {
            quotes.push(Quote {
                ticker: c.ticker.clone(),
                value: r.value,
                change: change(Subject::Comparable(c.id), r.value),
            });
        }
    }
    ticker_strip::show(ui, p, &quotes);
    ui.add_space(space::LG);

    let width = ui.available_width();
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::MD;
        let spotlight_width = SPOTLIGHT_WIDTH.min(width * 0.36);
        let headline_width = width - spotlight_width - space::MD;
        card(ui, p, headline_width, |ui| {
            headline(ui, p, reading, loaded, change(Subject::Own, reading.psi))
        });
        card(ui, p, spotlight_width, |ui| spotlight_monitor(ui, p, now));
    });
    ui.add_space(space::MD);

    card(ui, p, width, |ui| {
        ui.horizontal(|ui| {
            super::overline(ui, p, copy::dashboard::HISTORY);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for range in Range::ALL.into_iter().rev() {
                    ui.selectable_value(&mut state.range, range, range.label());
                }
            });
        });
        ui.add_space(space::SM);
        if loaded.candles.is_empty() {
            ui.add_space(space::LG);
            ui.label(RichText::new(copy::dashboard::NO_HISTORY).color(p.text_secondary));
            ui.add_space(space::LG);
        } else {
            chart::candlesticks(ui, p, &loaded.candles, loaded.threshold, CHART_HEIGHT);
        }
    });
    ui.add_space(space::MD);

    if width >= 900.0 {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::MD;
            let half = (width - space::MD) / 2.0;
            card(ui, p, half, |ui| positions_held(ui, p, loaded));
            card(ui, p, half, |ui| peer_summary(ui, p, loaded, reading));
        });
    } else {
        card(ui, p, width, |ui| positions_held(ui, p, loaded));
        ui.add_space(space::MD);
        card(ui, p, width, |ui| peer_summary(ui, p, loaded, reading));
    }
    ui.add_space(space::MD);

    card(ui, p, width, |ui| recent_notices(ui, p, loaded));
    ui.add_space(space::XL);

    // The Spotlight Monitor samples once per second.
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs(1));
}

fn load(
    session: &Session,
    range: Range,
    today: jiff::civil::Date,
    tz: &TimeZone,
) -> db::Result<Loaded> {
    let conn = &session.conn;
    let tomorrow = today.tomorrow().expect("date in range");
    let first_recorded =
        db::snapshots::first_at(conn, Subject::Own)?.map(|t| market::day_of(t, tz));
    let candles = match first_recorded {
        None => Vec::new(),
        Some(first_recorded) => {
            let first = match range.days() {
                Some(days) => first_recorded.max(today.saturating_sub((days - 1).days())),
                None => first_recorded,
            };
            let start = market::start_of(first, tz);
            let points =
                db::snapshots::between(conn, Subject::Own, start, market::start_of(tomorrow, tz))?;
            let prior = db::snapshots::last_before(conn, Subject::Own, start)?;
            market::daily_candles(&points, tz, first, today, prior)
        }
    };

    let comparables = db::comparables::active(conn)?;
    let start_today = market::start_of(today, tz);
    let end_today = market::start_of(tomorrow, tz);
    let mut reference = HashMap::new();
    let mut own_reference_is_close = false;
    for subject in
        std::iter::once(Subject::Own).chain(comparables.iter().map(|c| Subject::Comparable(c.id)))
    {
        let previous = db::snapshots::last_before(conn, subject, start_today)?;
        let opening = || -> db::Result<Option<f64>> {
            Ok(
                db::snapshots::between(conn, subject, start_today, end_today)?
                    .first()
                    .map(|(_, v)| *v),
            )
        };
        if subject == Subject::Own {
            own_reference_is_close = previous.is_some();
        }
        if let Some(v) = previous.map_or_else(opening, |v| Ok(Some(v)))? {
            reference.insert(subject, v);
        }
    }

    Ok(Loaded {
        revision: session.revision,
        range,
        today,
        candles,
        threshold: db::threshold::current(conn)?,
        reference,
        own_reference_is_close,
        comparables,
        names: db::dimensions::visible(conn)?
            .into_iter()
            .map(|d| (d.id, d.name))
            .collect(),
        positions: positions::held(&run::inputs(conn)?),
        notices: db::notices::recent(conn, NOTICES_SHOWN)?,
    })
}

fn card(ui: &mut Ui, p: &Palette, outer_width: f32, add: impl FnOnce(&mut Ui)) {
    theme::card(p).show(ui, |ui| {
        let inner = outer_width - 2.0 * space::LG - 2.0;
        ui.set_width(inner);
        ui.vertical(|ui| {
            ui.set_width(inner);
            add(ui);
        });
    });
}

fn headline(
    ui: &mut Ui,
    p: &Palette,
    reading: &crate::engine::index::Reading,
    loaded: &Loaded,
    change: Option<f64>,
) {
    super::overline(ui, p, copy::dashboard::PSI);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format::index(reading.psi))
                .family(theme::family::display())
                .size(64.0),
        );
        ui.add_space(space::SM);
        ui.vertical(|ui| {
            ui.add_space(space::LG);
            match change {
                Some(change) => {
                    let reference = reading.psi - change;
                    let pct = if reference.abs() > f64::EPSILON {
                        100.0 * change / reference.abs()
                    } else {
                        0.0
                    };
                    ui.label(
                        RichText::new(copy::dashboard::change(change, pct))
                            .family(theme::family::mono_medium())
                            .size(15.0)
                            .color(movement_color(p, change)),
                    );
                    ui.label(
                        RichText::new(if loaded.own_reference_is_close {
                            copy::dashboard::SINCE_CLOSE
                        } else {
                            copy::dashboard::SINCE_OPEN
                        })
                        .small()
                        .color(p.text_secondary),
                    );
                }
                None => {
                    ui.label(RichText::new(copy::dashboard::NO_CLOSE).color(p.text_secondary));
                }
            }
        });
    });
    ui.add_space(space::SM);
    let none = || copy::comparables::NO_VALUE.to_owned();
    let mut stats = vec![
        (
            copy::dashboard::COMPOSITE,
            reading
                .own_composite
                .map(format::index)
                .unwrap_or_else(none),
        ),
        (copy::dashboard::MOMENTUM, format::signed(reading.momentum)),
    ];
    if let Some(t) = loaded.threshold {
        stats.push((copy::dashboard::THRESHOLD, format::index(t)));
        stats.push((
            copy::dashboard::DISTANCE,
            format::index((t - reading.psi).max(0.0)),
        ));
    }
    let columns = if ui.available_width() >= 500.0 { 4 } else { 2 };
    egui::Grid::new("headline-stats")
        .num_columns(columns)
        .spacing(egui::vec2(space::LG, space::SM))
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            for (i, (label, value)) in stats.into_iter().enumerate() {
                ui.vertical(|ui| {
                    ui.label(super::overline_text(p, label));
                    ui.label(RichText::new(value).monospace());
                });
                if i % columns == columns - 1 {
                    ui.end_row();
                }
            }
        });
}

fn spotlight_monitor(ui: &mut Ui, p: &Palette, now: Timestamp) {
    let r = spotlight::read(now);
    super::overline(ui, p, copy::dashboard::SPOTLIGHT);
    ui.add_space(space::XS);
    ui.label(copy::dashboard::SPOTLIGHT_LABEL);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format::thousands(r.count))
                .family(theme::family::display())
                .size(48.0),
        );
        ui.label(
            RichText::new(copy::dashboard::margin(r.margin))
                .family(theme::family::mono_medium())
                .color(p.text_secondary),
        );
    });
    ui.label(
        RichText::new(copy::dashboard::last_nonzero(
            r.last_nonzero.map(|t| t.to_zoned(TimeZone::system())),
        ))
        .small()
        .color(p.text_secondary),
    );
    ui.label(
        RichText::new(copy::dashboard::reading_taken(
            &r.taken_at.to_zoned(TimeZone::system()),
        ))
        .small()
        .color(p.text_secondary),
    );
}

fn positions_held(ui: &mut Ui, p: &Palette, loaded: &Loaded) {
    super::overline(ui, p, copy::dashboard::POSITIONS);
    ui.add_space(space::SM);
    if loaded.positions.is_empty() {
        ui.label(RichText::new(copy::dashboard::NO_POSITIONS).color(p.text_secondary));
        return;
    }
    let name_width = (ui.available_width() - 200.0).max(120.0);
    for pos in loaded.positions.iter().take(8) {
        let name = loaded
            .names
            .get(&pos.dimension_id)
            .map_or(pos.dimension_id.as_str(), String::as_str);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(name_width, 20.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_width(name_width);
                    ui.add(egui::Label::new(name).truncate());
                },
            );
            ui.add_sized(
                [56.0, 20.0],
                egui::Label::new(RichText::new(format::signed(pos.margin)).monospace()),
            );
            ui.label(
                RichText::new(copy::dashboard::volatility(pos.volatility))
                    .small()
                    .color(p.text_secondary),
            );
        });
    }
    if loaded.positions.len() > 8 {
        ui.label(
            RichText::new(copy::dashboard::more_positions(loaded.positions.len() - 8))
                .small()
                .color(p.text_secondary),
        );
    }
}

fn peer_summary(
    ui: &mut Ui,
    p: &Palette,
    loaded: &Loaded,
    reading: &crate::engine::index::Reading,
) {
    super::overline(ui, p, copy::dashboard::PEERS);
    ui.add_space(space::SM);
    if loaded.comparables.is_empty() {
        ui.label(RichText::new(copy::comparables::EMPTY_TITLE).color(p.text_secondary));
        return;
    }
    egui::Grid::new("peer-summary")
        .num_columns(4)
        .spacing(egui::vec2(space::MD, space::XS))
        .show(ui, |ui| {
            for head in [
                copy::comparables::COL_TICKER,
                copy::comparables::COL_INDEX,
                copy::dashboard::COL_CHANGE,
                copy::comparables::COL_SPREAD,
            ] {
                ui.label(super::overline_text(p, head));
            }
            ui.end_row();
            for c in &loaded.comparables {
                let Some(r) = reading.comparable(c.id) else {
                    continue;
                };
                ui.label(RichText::new(&c.ticker).family(theme::family::mono_medium()))
                    .on_hover_text(&c.name);
                ui.label(RichText::new(format::index(r.value)).monospace());
                let change = loaded
                    .reference
                    .get(&Subject::Comparable(c.id))
                    .map(|v| r.value - v)
                    .unwrap_or(0.0);
                ui.label(
                    RichText::new(format::signed(change))
                        .monospace()
                        .color(movement_color(p, change)),
                );
                ui.label(
                    RichText::new(
                        reading
                            .spread(c.id)
                            .map(format::signed)
                            .unwrap_or_else(|| copy::comparables::NO_VALUE.to_owned()),
                    )
                    .monospace(),
                );
                ui.end_row();
            }
        });
}

fn recent_notices(ui: &mut Ui, p: &Palette, loaded: &Loaded) {
    super::overline(ui, p, copy::dashboard::NOTICES);
    ui.add_space(space::SM);
    if loaded.notices.is_empty() {
        ui.label(RichText::new(copy::dashboard::NO_NOTICES).color(p.text_secondary));
        return;
    }
    for n in &loaded.notices {
        ui.horizontal_top(|ui| {
            ui.add_sized(
                [96.0, 18.0],
                egui::Label::new(
                    RichText::new(
                        format::local_date(&n.created_at)
                            .map(format::date_short)
                            .unwrap_or_default(),
                    )
                    .small()
                    .color(p.text_secondary),
                ),
            );
            ui.add(egui::Label::new(&n.body).wrap());
        });
    }
}
