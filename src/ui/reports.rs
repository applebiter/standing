//! Reports: the Quarterly Earnings Call, shown as a printed page and exported
//! as Markdown or HTML.

use egui::{RichText, Sense, Ui};
use jiff::tz::TimeZone;

use super::Notice;
use crate::app::Session;
use crate::copy;
use crate::engine::market;
use crate::report::earnings_call::{self, EarningsCall, Figure, Quarter, Role};
use crate::report::{self, Format, data};
use crate::theme::{self, Palette, space};

const PAGE_WIDTH: f32 = 720.0;

struct Loaded {
    revision: u64,
    quarter: Quarter,
    call: EarningsCall,
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
                        call: earnings_call::prepare(&d),
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
            .width(180.0)
            .selected_text(
                state
                    .selected
                    .map_or_else(String::new, |q| copy::reports::quarter_option(&q.label())),
            )
            .show_ui(ui, |ui| {
                for q in &state.quarters {
                    ui.selectable_value(
                        &mut state.selected,
                        Some(*q),
                        copy::reports::quarter_option(&q.label()),
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
                    state.notice = export(&l.call, format);
                }
            }
        }
    });
    ui.add_space(space::MD);
    super::notice(ui, p, &mut state.notice);

    if let Some(l) = &state.loaded {
        page(ui, p, &l.call);
    }
    ui.add_space(space::XL);
}

/// Asks where to save, and writes the report there.
fn export(call: &EarningsCall, format: Format) -> Option<Notice> {
    let contents = match report::render(call, format) {
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
        .set_file_name(report::file_name(call, format))
        .add_filter(label, &[ext])
        .save_file()?;
    Some(match report::write(&path, &contents) {
        Ok(()) => Notice::Info(copy::reports::exported(&path.display().to_string())),
        Err(e) => Notice::Error(format!("{} {e}", copy::reports::EXPORT_FAILED)),
    })
}

fn page(ui: &mut Ui, p: &Palette, call: &EarningsCall) {
    let width = ui.available_width().min(PAGE_WIDTH);
    egui::Frame::new()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .inner_margin(egui::Margin::symmetric(48, 40))
        .show(ui, |ui| {
            ui.set_width(width - 96.0);
            super::overline(ui, p, copy::reports::TRANSCRIPT);
            ui.add_space(space::XS);
            ui.label(
                RichText::new(&call.title)
                    .family(theme::family::display())
                    .size(36.0),
            );
            ui.add_space(space::XS);
            ui.label(
                RichText::new(format!("{} \u{b7} {}", call.company, call.period))
                    .monospace()
                    .size(12.0)
                    .color(p.text_secondary),
            );

            heading(ui, p, copy::reports::HIGHLIGHTS);
            rows(ui, p, &call.figures);

            for section in &call.sections {
                heading(ui, p, &section.title);
                for turn in &section.turns {
                    let speaker =
                        RichText::new(&turn.speaker).family(theme::family::sans_semibold());
                    ui.label(match turn.role {
                        Role::Analyst => speaker.color(p.text_secondary),
                        _ => speaker,
                    });
                    for para in &turn.paragraphs {
                        let text = RichText::new(para).size(15.0);
                        ui.add(
                            egui::Label::new(match turn.role {
                                Role::Operator => text.italics().color(p.text_secondary),
                                _ => text,
                            })
                            .wrap(),
                        );
                    }
                    ui.add_space(space::MD);
                }
            }
            if !call.headwinds.is_empty() {
                heading(ui, p, copy::reports::SUPPLEMENT_HEADWINDS);
                ui.label(
                    RichText::new(copy::reports::SPREAD_NOTE)
                        .small()
                        .color(p.text_secondary),
                );
                ui.add_space(space::XS);
                rows(ui, p, &call.headwinds);
            }
            if !call.filings.is_empty() {
                heading(ui, p, copy::reports::SUPPLEMENT_FILINGS);
                rows(ui, p, &call.filings);
            }
        });
}

fn heading(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add_space(space::LG);
    super::overline(ui, p, text);
    ui.add_space(space::XS);
    rule(ui, p);
    ui.add_space(space::SM);
}

fn rule(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, p.border),
    );
}

fn rows(ui: &mut Ui, p: &Palette, figures: &[Figure]) {
    ui.spacing_mut().interact_size.y = 0.0;
    for f in figures {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&f.label).color(p.text_secondary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&f.value).monospace());
            });
        });
        ui.add_space(space::XS);
        rule(ui, p);
        ui.add_space(space::XS);
    }
}
