//! Ratings: assess yourself and each Comparable on every applicable dimension.
//!
//! A slider writes one rating when it is released (or changed from the
//! keyboard), never on every frame of a drag.

use std::collections::HashMap;

use egui::{Align, Align2, Layout, RichText, Sense, Ui};
use jiff::Timestamp;

use crate::app::Session;
use crate::db;
use crate::model::comparable::{Comparable, SELF_TICKER};
use crate::model::dimension::Dimension;
use crate::model::rating::{self, Rating, Subject};
use crate::theme::{self, Palette, space};
use crate::{copy, format};

/// Sector tree width: a share of the screen, within these bounds.
const TREE_WIDTH: std::ops::RangeInclusive<f32> = 200.0..=272.0;
/// Ticker and name column on each dimension card, within these bounds.
const LABEL_WIDTH: std::ops::RangeInclusive<f32> = 130.0..=180.0;
/// Room for the value and the provisional marker.
const VALUE_WIDTH: f32 = 96.0;

type Key = (Subject, String);

struct Loaded {
    revision: u64,
    /// Applicable dimensions, in catalog order.
    dimensions: Vec<Dimension>,
    comparables: Vec<Comparable>,
    current: HashMap<Key, Rating>,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    /// Selected (sector, industry).
    selected: Option<(String, String)>,
    search: String,
    /// Values being dragged, not yet recorded.
    drafts: HashMap<Key, f64>,
    error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::ratings::TITLE, copy::ratings::SUBTITLE);

    if state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision)
    {
        match load(session) {
            Ok(loaded) => state.loaded = Some(loaded),
            Err(e) => state.error = Some(format!("{} {e:#}", copy::ratings::LOAD_FAILED)),
        }
    }
    if let Some(err) = &state.error {
        ui.label(RichText::new(err).color(p.danger));
        ui.add_space(space::MD);
    }
    let Some(loaded) = &mut state.loaded else {
        return;
    };
    if loaded.dimensions.is_empty() {
        ui.label(RichText::new(copy::ratings::NO_DIMENSIONS).color(p.text_secondary));
        return;
    }
    if state.selected.is_none() {
        let d = &loaded.dimensions[0];
        state.selected = Some((d.sector.clone(), d.industry.clone()));
    }

    egui::Panel::left("ratings-tree")
        .resizable(false)
        .exact_size((ui.available_width() * 0.32).clamp(*TREE_WIDTH.start(), *TREE_WIDTH.end()))
        .frame(egui::Frame::new().inner_margin(egui::Margin {
            right: space::LG as i8,
            ..Default::default()
        }))
        .show(ui, |ui| {
            tree(ui, p, loaded, &mut state.selected, &mut state.search)
        });

    let searching = !state.search.trim().is_empty();
    let shown: Vec<usize> = if searching {
        let needle = state.search.trim().to_lowercase();
        (0..loaded.dimensions.len())
            .filter(|&i| {
                let d = &loaded.dimensions[i];
                d.name.to_lowercase().contains(&needle) || d.id.to_lowercase().contains(&needle)
            })
            .collect()
    } else {
        let (sector, industry) = state.selected.clone().unwrap_or_default();
        (0..loaded.dimensions.len())
            .filter(|&i| {
                let d = &loaded.dimensions[i];
                d.sector == sector && d.industry == industry
            })
            .collect()
    };

    let scroll_id = if searching {
        ("ratings-search".to_owned(), String::new())
    } else {
        state.selected.clone().unwrap_or_default()
    };
    egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink(false)
        .show(ui, |ui| {
            if searching {
                ui.label(
                    RichText::new(copy::ratings::search_results(shown.len()))
                        .small()
                        .color(p.text_secondary),
                );
                ui.add_space(space::SM);
            } else if let Some((sector, industry)) = &state.selected {
                super::overline(ui, p, sector);
                ui.label(
                    RichText::new(industry)
                        .family(theme::family::serif())
                        .size(20.0),
                );
                ui.add_space(space::MD);
            }
            if loaded.comparables.is_empty() {
                ui.label(RichText::new(copy::ratings::NO_COMPARABLES).color(p.text_secondary));
                ui.add_space(space::MD);
            }
            for i in shown {
                let committed = dimension_card(ui, p, loaded, i, &mut state.drafts);
                for (subject, value) in committed {
                    let id = loaded.dimensions[i].id.clone();
                    match db::ratings::insert(&session.conn, subject, &id, value, Timestamp::now())
                    {
                        Ok(r) => {
                            loaded.current.insert((subject, id), r);
                            session.revision += 1;
                            loaded.revision = session.revision;
                            state.error = None;
                        }
                        Err(e) => {
                            state.error = Some(format!("{} {e:#}", copy::ratings::WRITE_FAILED))
                        }
                    }
                }
                ui.add_space(space::SM);
            }
            ui.add_space(space::XL);
        });
}

fn load(session: &Session) -> db::Result<Loaded> {
    Ok(Loaded {
        revision: session.revision,
        dimensions: db::dimensions::visible(&session.conn)?
            .into_iter()
            .filter(Dimension::is_active)
            .collect(),
        comparables: db::comparables::active(&session.conn)?,
        current: db::ratings::current(&session.conn)?,
    })
}

/// A sector in the tree, in catalog order.
struct SectorEntry<'a> {
    name: &'a str,
    industries: Vec<IndustryEntry<'a>>,
}

/// An industry, with how many of its dimensions the user has rated themselves on.
struct IndustryEntry<'a> {
    name: &'a str,
    total: usize,
    rated: usize,
}

/// Sector and industry navigation, with self-rating coverage per industry.
fn tree(
    ui: &mut Ui,
    p: &Palette,
    loaded: &Loaded,
    selected: &mut Option<(String, String)>,
    search: &mut String,
) {
    ui.add(
        egui::TextEdit::singleline(search)
            .hint_text(copy::ratings::SEARCH_HINT)
            .desired_width(f32::INFINITY),
    );
    ui.add_space(space::MD);

    // Sectors and industries in order of first appearance in the catalog. An
    // industry's dimensions need not be adjacent: identifiers are appended
    // per sector as the catalog grows.
    let mut sectors: Vec<SectorEntry<'_>> = Vec::new();
    for d in &loaded.dimensions {
        let sector = match sectors.iter().position(|s| s.name == d.sector) {
            Some(i) => &mut sectors[i],
            None => {
                sectors.push(SectorEntry {
                    name: &d.sector,
                    industries: Vec::new(),
                });
                sectors.last_mut().expect("just pushed")
            }
        };
        let industry = match sector.industries.iter().position(|i| i.name == d.industry) {
            Some(i) => &mut sector.industries[i],
            None => {
                sector.industries.push(IndustryEntry {
                    name: &d.industry,
                    total: 0,
                    rated: 0,
                });
                sector.industries.last_mut().expect("just pushed")
            }
        };
        industry.total += 1;
        if loaded.current.contains_key(&(Subject::Own, d.id.clone())) {
            industry.rated += 1;
        }
    }

    egui::ScrollArea::vertical()
        .id_salt("ratings-tree-scroll")
        .auto_shrink(false)
        .show(ui, |ui| {
            for SectorEntry {
                name: sector,
                industries,
            } in sectors
            {
                let open = selected.as_ref().is_some_and(|(s, _)| s == sector);
                egui::CollapsingHeader::new(
                    RichText::new(sector).family(theme::family::sans_medium()),
                )
                .id_salt(("sector", sector))
                .default_open(open)
                .show(ui, |ui| {
                    for IndustryEntry {
                        name: industry,
                        total,
                        rated,
                    } in industries
                    {
                        let is_selected = selected
                            .as_ref()
                            .is_some_and(|(s, i)| s == sector && i == industry);
                        if tree_item(
                            ui,
                            p,
                            industry,
                            &copy::ratings::rated(rated, total),
                            is_selected,
                        ) {
                            *selected = Some((sector.to_owned(), industry.to_owned()));
                            search.clear();
                        }
                    }
                });
            }
        });
}

fn tree_item(ui: &mut Ui, p: &Palette, label: &str, count: &str, selected: bool) -> bool {
    let height = 28.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());
    if selected {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.5),
        );
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(2.0, height));
        ui.painter().rect_filled(bar, 0.0, p.accent);
    } else if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.25),
        );
    }
    let body = egui::FontId::new(13.5, egui::FontFamily::Proportional);
    ui.painter().text(
        rect.left_center() + egui::vec2(space::SM + 2.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        body,
        if selected {
            p.text_primary
        } else {
            p.text_secondary
        },
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(space::SM, 0.0),
        Align2::RIGHT_CENTER,
        count,
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
        p.text_secondary,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// One dimension, with a slider per subject. Returns ratings to record.
fn dimension_card(
    ui: &mut Ui,
    p: &Palette,
    loaded: &Loaded,
    index: usize,
    drafts: &mut HashMap<Key, f64>,
) -> Vec<(Subject, f64)> {
    let d = &loaded.dimensions[index];
    let mut committed = Vec::new();
    let inner_width = ui.available_width() - 2.0 * space::LG - 2.0;
    let label_width = (inner_width * 0.3).clamp(*LABEL_WIDTH.start(), *LABEL_WIDTH.end());
    let slider_width =
        (inner_width - label_width - VALUE_WIDTH - 2.0 * space::MD).clamp(96.0, 360.0);
    theme::card(p)
        .inner_margin(egui::Margin::symmetric(space::LG as i8, space::MD as i8))
        .show(ui, |ui| {
            ui.set_width(inner_width);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(&d.id).monospace().color(p.text_secondary));
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(&d.name)
                                    .family(theme::family::sans_medium())
                                    .size(15.0),
                            )
                            .truncate(),
                        );
                    });
                });
            });
            ui.label(RichText::new(&d.definition).small().color(p.text_secondary));
            ui.add_space(space::SM);

            let subjects = std::iter::once((
                Subject::Own,
                copy::ratings::SELF_LABEL.to_owned(),
                SELF_TICKER.to_owned(),
            ))
            .chain(
                loaded
                    .comparables
                    .iter()
                    .map(|c| (Subject::Comparable(c.id), c.name.clone(), c.ticker.clone())),
            );
            egui::Grid::new(("rating-grid", &d.id))
                .num_columns(3)
                .spacing(egui::vec2(space::MD, space::XS))
                .show(ui, |ui| {
                    for (subject, name, ticker) in subjects {
                        ui.horizontal(|ui| {
                            ui.set_width(label_width);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&ticker)
                                        .family(theme::family::mono_medium())
                                        .size(12.0)
                                        .color(p.text_secondary),
                                )
                                .selectable(false),
                            );
                            ui.add(egui::Label::new(name).truncate().selectable(false));
                        });
                        let key = (subject, d.id.clone());
                        let current = loaded.current.get(&key);
                        if let Some(value) = slider(ui, p, slider_width, current, drafts, key) {
                            committed.push((subject, value));
                        }
                        let value = drafts
                            .get(&(subject, d.id.clone()))
                            .copied()
                            .or(current.map(|r| r.value));
                        ui.horizontal(|ui| match (value, current) {
                            (Some(v), Some(r))
                                if r.provisional
                                    && !drafts.contains_key(&(subject, d.id.clone())) =>
                            {
                                ui.label(RichText::new(format::rating(v)).monospace().italics());
                                ui.label(
                                    RichText::new(copy::ratings::PROVISIONAL)
                                        .small()
                                        .italics()
                                        .color(p.text_secondary),
                                );
                            }
                            (Some(v), _) => {
                                ui.label(RichText::new(format::rating(v)).monospace());
                            }
                            (None, _) => {
                                ui.label(
                                    RichText::new(copy::ratings::NOT_RATED)
                                        .small()
                                        .color(p.text_secondary),
                                );
                            }
                        });
                        ui.end_row();
                    }
                });
        });
    committed
}

/// A 0–10 slider that reports a value to record once the pointer is released,
/// or immediately for keyboard changes.
fn slider(
    ui: &mut Ui,
    p: &Palette,
    width: f32,
    current: Option<&Rating>,
    drafts: &mut HashMap<Key, f64>,
    key: Key,
) -> Option<f64> {
    let rated = current.is_some();
    let mut value = drafts
        .get(&key)
        .copied()
        .or(current.map(|r| r.value))
        .unwrap_or((rating::MIN + rating::MAX) / 2.0);
    let set = rated || drafts.contains_key(&key);
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = width;
            ui.spacing_mut().slider_rail_height = 4.0;
            let w = &mut ui.visuals_mut().widgets;
            w.inactive.bg_fill = p.border;
            if !set {
                w.inactive.fg_stroke.color = p.text_secondary;
            }
            ui.add(
                egui::Slider::new(&mut value, rating::MIN..=rating::MAX)
                    .step_by(rating::STEP)
                    .show_value(false)
                    .handle_shape(egui::style::HandleShape::Circle)
                    .trailing_fill(set),
            )
        })
        .inner;
    if response.is_pointer_button_down_on() {
        if response.changed() || drafts.contains_key(&key) {
            drafts.insert(key, value);
        }
        None
    } else if let Some(v) = drafts.remove(&key) {
        (!unchanged(current, v)).then_some(v)
    } else if response.changed() {
        (!unchanged(current, value)).then_some(value)
    } else {
        None
    }
}

/// Whether recording `value` would change nothing: same value, already confirmed.
fn unchanged(current: Option<&Rating>, value: f64) -> bool {
    current.is_some_and(|r| !r.provisional && r.value == value)
}
