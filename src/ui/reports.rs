//! Reports: the Quarterly Statement, shown as a printed page and exported as
//! Markdown or HTML.

use egui::{Align, Layout, RichText, Sense, Ui};
use jiff::tz::TimeZone;

use super::Notice;
use crate::app::Session;
use crate::copy;
use crate::engine::market;
use crate::report::statement::{self, Quarter, Section, Statement};
use crate::report::{self, Format, data};
use crate::theme::{self, Palette, space};

const PAGE_WIDTH: f32 = 760.0;

struct Loaded {
    revision: u64,
    quarter: Quarter,
    statement: Statement,
}

#[derive(Default)]
pub struct State {
    quarters: Vec<Quarter>,
    selected: Option<Quarter>,
    loaded: Option<Loaded>,
    notice: Option<Notice>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::reports::TITLE, copy::reports::SUBTITLE);
    let tz = TimeZone::system();
    let today = market::day_of(jiff::Timestamp::now(), &tz);

    let stale = state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision || Some(l.quarter) != state.selected);
    if stale {
        match data::quarters(&session.conn, today, &tz) {
            Ok(qs) => state.quarters = qs,
            Err(e) => {
                state.notice = Some(Notice::Error(format!(
                    "{} {e:#}",
                    copy::reports::LOAD_FAILED
                )));
            }
        }
        let quarter = state
            .selected
            .filter(|q| state.quarters.contains(q))
            .or_else(|| state.quarters.first().copied());
        state.selected = quarter;
        if let Some(quarter) = quarter {
            match data::gather(&session.conn, quarter, today, &tz) {
                Ok(d) => {
                    state.loaded = Some(Loaded {
                        revision: session.revision,
                        quarter,
                        statement: statement::prepare(&d),
                    })
                }
                Err(e) => {
                    state.notice = Some(Notice::Error(format!(
                        "{} {e:#}",
                        copy::reports::LOAD_FAILED
                    )))
                }
            }
        }
    }

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("report-quarter")
            .width(240.0)
            .selected_text(
                state
                    .selected
                    .map_or_else(String::new, |q| copy::statement::title(&q.label())),
            )
            .show_ui(ui, |ui| {
                for q in &state.quarters {
                    ui.selectable_value(
                        &mut state.selected,
                        Some(*q),
                        copy::statement::title(&q.label()),
                    );
                }
            });
        ui.add_space(space::SM);
        if let Some(l) = &state.loaded {
            for (format, label) in [
                (Format::Markdown, copy::reports::EXPORT_MARKDOWN),
                (Format::Html, copy::reports::EXPORT_HTML),
            ] {
                if super::secondary_button(ui, label).clicked() {
                    state.notice = export(&l.statement, format);
                }
            }
        }
    });
    ui.add_space(space::MD);
    super::notice(ui, p, &mut state.notice);

    if let Some(l) = &state.loaded {
        page(ui, p, &l.statement);
    }
    ui.add_space(space::XL);
}

/// Asks where to save, and writes the statement there.
fn export(statement: &Statement, format: Format) -> Option<Notice> {
    let contents = match report::render(statement, format) {
        Ok(c) => c,
        Err(e) => {
            return Some(Notice::Error(format!(
                "{} {e:#}",
                copy::reports::EXPORT_FAILED
            )));
        }
    };
    let (label, ext) = match format {
        Format::Markdown => (copy::reports::FILTER_MARKDOWN, "md"),
        Format::Html => (copy::reports::FILTER_HTML, "html"),
    };
    let path = rfd::FileDialog::new()
        .set_file_name(report::file_name(statement, format))
        .add_filter(label, &[ext])
        .save_file()?;
    Some(match report::write(&path, &contents) {
        Ok(()) => Notice::Info(copy::reports::exported(&path.display().to_string())),
        Err(e) => Notice::Error(format!("{} {e}", copy::reports::EXPORT_FAILED)),
    })
}

fn page(ui: &mut Ui, p: &Palette, s: &Statement) {
    let width = ui.available_width().min(PAGE_WIDTH);
    egui::Frame::new()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .inner_margin(egui::Margin::symmetric(48, 40))
        .show(ui, |ui| {
            ui.set_width(width - 98.0);
            super::overline(ui, p, copy::reports::STATEMENT);
            ui.add_space(space::XS);
            ui.label(
                RichText::new(&s.title)
                    .family(theme::family::display())
                    .size(34.0),
            );
            ui.add_space(space::XS);
            ui.label(
                RichText::new(format!("{} \u{b7} {}", s.holder, s.period))
                    .monospace()
                    .size(12.0)
                    .color(p.text_secondary),
            );
            for section in &s.sections {
                table(ui, p, section);
            }
        });
}

fn rule(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, p.border),
    );
}

/// One section: a heading, an optional note, and its table (or the empty
/// message). The first column takes the remaining width.
fn table(ui: &mut Ui, p: &Palette, section: &Section) {
    ui.add_space(space::LG);
    super::overline(ui, p, &section.title);
    ui.add_space(space::XS);
    rule(ui, p);
    ui.add_space(space::SM);
    if let Some(note) = &section.note {
        ui.add(egui::Label::new(RichText::new(note).small().color(p.text_secondary)).wrap());
        ui.add_space(space::XS);
    }
    if section.rows.is_empty() {
        ui.label(RichText::new(&section.empty).color(p.text_secondary));
        return;
    }
    let width = ui.available_width();
    let col_width = |i: usize| width * section.columns[i].width;
    ui.spacing_mut().interact_size.y = 0.0;
    let row = |ui: &mut Ui, cells: Vec<RichText>| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, cell) in cells.into_iter().enumerate() {
                let layout = if section.columns[i].numeric {
                    Layout::right_to_left(Align::Center)
                } else {
                    Layout::left_to_right(Align::Center)
                };
                ui.allocate_ui_with_layout(egui::vec2(col_width(i), 20.0), layout, |ui| {
                    ui.set_width(col_width(i));
                    // A gutter between columns.
                    if i > 0 && !section.columns[i].numeric {
                        ui.add_space(space::MD);
                    }
                    ui.add(egui::Label::new(cell).truncate());
                });
            }
        });
    };
    row(
        ui,
        section
            .columns
            .iter()
            .map(|c| super::overline_text(p, &c.label))
            .collect(),
    );
    ui.add_space(space::XS);
    for cells in &section.rows {
        row(
            ui,
            cells
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let t = RichText::new(c);
                    if section.columns[i].numeric {
                        t.monospace()
                    } else {
                        t
                    }
                })
                .collect(),
        );
        ui.add_space(space::XS);
        rule(ui, p);
        ui.add_space(space::XS);
    }
}
