//! Notices: market closes, threshold adjustments, peer set changes and
//! discoveries, newest first.

use egui::{RichText, Ui};
use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::app::Session;
use crate::db::{self, notices::Notice};
use crate::theme::{self, Palette, space};
use crate::{copy, format};

/// Notices listed; older ones are kept but not shown.
const SHOWN: usize = 500;

struct Loaded {
    revision: u64,
    notices: Vec<Notice>,
}

#[derive(Default)]
pub struct State {
    loaded: Option<Loaded>,
    kind: Option<&'static str>,
    error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(
        ui,
        p,
        copy::notices_screen::TITLE,
        copy::notices_screen::SUBTITLE,
    );
    if state
        .loaded
        .as_ref()
        .is_none_or(|l| l.revision != session.revision)
    {
        match db::notices::recent(&session.conn, SHOWN) {
            Ok(notices) => {
                state.loaded = Some(Loaded {
                    revision: session.revision,
                    notices,
                });
                state.error = None;
            }
            Err(e) => state.error = Some(format!("{} {e:#}", copy::notices_screen::LOAD_FAILED)),
        }
    }
    // Seen now. Rows keep their "New" mark until the list is next loaded.
    if session.unread_notices > 0
        && db::notices::mark_all_read(&session.conn, Timestamp::now()).is_ok()
    {
        session.unread_notices = 0;
    }
    if let Some(err) = &state.error {
        ui.label(RichText::new(err).color(p.danger));
        ui.add_space(space::MD);
    }
    let Some(loaded) = &state.loaded else {
        return;
    };

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("notice-kind")
            .width(180.0)
            .selected_text(
                state
                    .kind
                    .map_or(copy::notices_screen::ALL, copy::notices_screen::kind_label),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.kind, None, copy::notices_screen::ALL);
                for k in copy::notices_screen::KINDS {
                    ui.selectable_value(
                        &mut state.kind,
                        Some(k),
                        copy::notices_screen::kind_label(k),
                    );
                }
            });
    });
    ui.add_space(space::MD);

    let shown: Vec<&Notice> = loaded
        .notices
        .iter()
        .filter(|n| state.kind.is_none_or(|k| n.kind == k))
        .collect();
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if shown.is_empty() {
            ui.label(RichText::new(copy::notices_screen::NONE).color(p.text_secondary));
            return;
        }
        let tz = TimeZone::system();
        for (i, n) in shown.iter().enumerate() {
            if i > 0 {
                ui.add_space(space::XS);
                let rect = ui.available_rect_before_wrap();
                ui.painter().hline(
                    rect.x_range(),
                    rect.top(),
                    egui::Stroke::new(1.0, p.border.gamma_multiply(0.6)),
                );
                ui.add_space(space::SM);
            }
            ui.horizontal_top(|ui| {
                let when = n
                    .created_at
                    .parse::<Timestamp>()
                    .map(|t| t.to_zoned(tz.clone()))
                    .ok();
                ui.vertical(|ui| {
                    ui.set_width(104.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if let Some(z) = &when {
                        ui.label(
                            RichText::new(format::date_short(z.date()))
                                .small()
                                .color(p.text_secondary),
                        );
                        ui.label(
                            RichText::new(z.strftime("%H:%M").to_string())
                                .monospace()
                                .color(p.text_secondary),
                        );
                    }
                });
                ui.vertical(|ui| {
                    ui.set_width(96.0);
                    ui.label(super::overline_text(
                        p,
                        copy::notices_screen::kind_label(&n.kind),
                    ));
                    if !n.read {
                        ui.label(
                            super::overline_text(p, copy::notices_screen::UNREAD).color(p.accent),
                        );
                    }
                });
                ui.add(egui::Label::new(&n.body).wrap());
            });
        }
    });
    ui.add_space(space::XL);
}
