//! The Dimension Catalog: browse, search and apply SCCD dimensions.

use std::collections::BTreeSet;

use egui::{Align, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::app::Session;
use crate::db::dimensions;
use crate::model::dimension::{Dimension, DimensionState};
use crate::theme::{self, Palette, space};
use crate::{copy, format};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    All,
    Applies,
    Excluded,
    Discovered,
}

impl Status {
    const ALL: [Self; 4] = [Self::All, Self::Applies, Self::Excluded, Self::Discovered];

    fn label(self) -> &'static str {
        match self {
            Self::All => copy::catalog::STATUS_ALL,
            Self::Applies => copy::catalog::STATUS_APPLIES,
            Self::Excluded => copy::catalog::STATUS_EXCLUDED,
            Self::Discovered => copy::catalog::STATUS_DISCOVERED,
        }
    }

    fn admits(self, d: &Dimension) -> bool {
        match self {
            Self::All => true,
            Self::Applies => d.applies,
            Self::Excluded => !d.applies,
            Self::Discovered => d.state == DimensionState::Discovered,
        }
    }
}

struct Filters {
    search: String,
    sector: Option<String>,
    status: Status,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            search: String::new(),
            sector: None,
            status: Status::All,
        }
    }
}

impl Filters {
    fn is_default(&self) -> bool {
        self.search.trim().is_empty() && self.sector.is_none() && self.status == Status::All
    }

    fn admits(&self, d: &Dimension) -> bool {
        let needle = self.search.trim().to_lowercase();
        self.sector.as_ref().is_none_or(|s| *s == d.sector)
            && self.status.admits(d)
            && (needle.is_empty()
                || [&d.id, &d.name, &d.definition, &d.industry]
                    .iter()
                    .any(|f| f.to_lowercase().contains(&needle)))
    }
}

#[derive(Default)]
pub struct State {
    dimensions: Option<Vec<Dimension>>,
    /// The session revision `dimensions` was loaded at.
    revision: u64,
    filters: Filters,
    error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::catalog::TITLE, &copy::catalog::subtitle());

    if state.dimensions.is_none() || state.revision != session.revision {
        match dimensions::visible(&session.conn) {
            Ok(d) => {
                state.dimensions = Some(d);
                state.revision = session.revision;
            }
            Err(e) => state.error = Some(format!("{} {e:#}", copy::catalog::LOAD_FAILED)),
        }
    }
    if let Some(err) = &state.error {
        ui.label(RichText::new(err).color(p.danger));
        ui.add_space(space::MD);
    }
    let Some(dims) = &mut state.dimensions else {
        return;
    };

    summary(ui, p, dims);
    ui.add_space(space::LG);

    let sectors: Vec<String> = dims
        .iter()
        .map(|d| d.sector.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let shown: Vec<usize> = (0..dims.len())
        .filter(|&i| state.filters.admits(&dims[i]))
        .collect();
    filter_bar(ui, p, &mut state.filters, &sectors, shown.len(), dims.len());
    ui.add_space(space::MD);

    if shown.is_empty() {
        ui.add_space(space::XL);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(copy::catalog::NO_MATCHES)
                    .family(theme::family::serif())
                    .size(22.0),
            );
        });
        return;
    }

    if let Some((id, applies)) = table(ui, p, dims, &shown) {
        match dimensions::set_applies(&session.conn, &id, applies) {
            Ok(()) => {
                if let Some(d) = dims.iter_mut().find(|d| d.id == id) {
                    d.applies = applies;
                }
                session.revision += 1;
                state.revision = session.revision;
                state.error = None;
            }
            Err(e) => state.error = Some(format!("{} {e:#}", copy::catalog::SAVE_FAILED)),
        }
    }
}

fn summary(ui: &mut Ui, p: &Palette, dims: &[Dimension]) {
    let active = dims.iter().filter(|d| d.is_active()).count();
    let sectors = dims
        .iter()
        .map(|d| &d.sector)
        .collect::<BTreeSet<_>>()
        .len();
    let industries = dims
        .iter()
        .map(|d| (&d.sector, &d.industry))
        .collect::<BTreeSet<_>>()
        .len();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::MD;
        for (label, value) in [
            (copy::catalog::ACTIVE_DIMENSIONS, active),
            (copy::catalog::SECTORS, sectors),
            (copy::catalog::INDUSTRIES, industries),
        ] {
            theme::card(p)
                .inner_margin(egui::Margin::symmetric(space::LG as i8, space::MD as i8))
                .show(ui, |ui| {
                    ui.set_min_width(150.0);
                    ui.vertical(|ui| {
                        super::overline(ui, p, label);
                        ui.label(
                            RichText::new(format::thousands(value as u64))
                                .family(theme::family::mono_medium())
                                .size(26.0),
                        );
                    });
                });
        }
    });
}

fn filter_bar(
    ui: &mut Ui,
    p: &Palette,
    filters: &mut Filters,
    sectors: &[String],
    shown: usize,
    total: usize,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut filters.search)
                .hint_text(copy::catalog::SEARCH_HINT)
                .desired_width(280.0),
        );
        egui::ComboBox::from_id_salt("sector-filter")
            .width(220.0)
            .selected_text(
                filters
                    .sector
                    .as_deref()
                    .unwrap_or(copy::catalog::ALL_SECTORS),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filters.sector, None, copy::catalog::ALL_SECTORS);
                for sector in sectors {
                    ui.selectable_value(&mut filters.sector, Some(sector.clone()), sector);
                }
            });
        egui::ComboBox::from_id_salt("status-filter")
            .width(150.0)
            .selected_text(filters.status.label())
            .show_ui(ui, |ui| {
                for status in Status::ALL {
                    ui.selectable_value(&mut filters.status, status, status.label());
                }
            });
    });
    ui.add_space(space::SM);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(copy::catalog::showing(shown, total))
                .small()
                .color(p.text_secondary),
        );
        if !filters.is_default() {
            ui.add_space(space::SM);
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(copy::catalog::CLEAR_FILTERS)
                            .small()
                            .color(p.accent),
                    )
                    .frame(false),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                *filters = Filters::default();
            }
        }
    });
}

/// Draws the table and returns a changed "Applies to me" toggle, if any.
fn table(ui: &mut Ui, p: &Palette, dims: &[Dimension], shown: &[usize]) -> Option<(String, bool)> {
    let wide = ui.available_width() >= 900.0;
    let mut changed = None;

    let mut builder = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(Layout::left_to_right(Align::Center))
        .auto_shrink([false, true])
        .column(Column::exact(84.0))
        .column(Column::remainder().at_least(240.0).clip(true))
        .column(Column::initial(170.0).clip(true));
    if wide {
        builder = builder.column(Column::initial(190.0).clip(true));
    }
    builder = builder
        .column(Column::exact(64.0))
        .column(Column::exact(64.0));

    builder
        .header(28.0, |mut header| {
            let mut head = |text: &str| {
                header.col(|ui| {
                    ui.label(super::overline_text(p, text));
                });
            };
            head(copy::catalog::COL_ID);
            head(copy::catalog::COL_DIMENSION);
            head(copy::catalog::COL_INDUSTRY);
            if wide {
                head(copy::catalog::COL_SECTOR);
            }
            head(copy::catalog::COL_WEIGHT);
            head(copy::catalog::COL_APPLIES);
        })
        .body(|body| {
            body.rows(50.0, shown.len(), |mut row| {
                let d = &dims[shown[row.index()]];
                let muted = !d.applies;
                let primary = if muted {
                    p.text_secondary
                } else {
                    p.text_primary
                };
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(RichText::new(&d.id).monospace().color(p.text_secondary))
                            .selectable(false),
                    );
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(RichText::new(&d.name).color(primary))
                                    .truncate()
                                    .selectable(false),
                            );
                            if d.state == DimensionState::Discovered {
                                ui.label(
                                    super::overline_text(p, copy::catalog::DISCOVERED_TAG)
                                        .color(p.accent),
                                );
                            }
                        });
                        ui.add(
                            egui::Label::new(
                                RichText::new(&d.definition).small().color(p.text_secondary),
                            )
                            .truncate()
                            .selectable(false),
                        );
                    });
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(RichText::new(&d.industry).color(p.text_secondary))
                            .truncate()
                            .selectable(false),
                    );
                });
                if wide {
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(RichText::new(&d.sector).color(p.text_secondary))
                                .truncate()
                                .selectable(false),
                        );
                    });
                }
                row.col(|ui| {
                    ui.label(
                        RichText::new(format!("{:.2}", d.weight))
                            .monospace()
                            .color(primary),
                    );
                });
                row.col(|ui| {
                    let mut applies = d.applies;
                    if ui
                        .checkbox(&mut applies, "")
                        .on_hover_text(copy::catalog::APPLIES_HINT)
                        .changed()
                    {
                        changed = Some((d.id.clone(), applies));
                    }
                });
            });
        });
    changed
}
