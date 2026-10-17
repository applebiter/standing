//! Screens and shared widgets. The only module tree that depends on egui
//! besides `app` and `theme`.

pub mod catalog;
pub mod chart;
pub mod comparable_form;
pub mod comparables;
pub mod daily_filing;
pub mod dashboard;
pub mod form;
pub mod nav;
pub mod pending;
pub mod ratings;
pub mod settings;
pub mod startup_error;
pub mod ticker_strip;

use egui::{Align, Button, Layout, Response, RichText, Sense, Ui};

use crate::theme::{self, Palette, space};

/// Per-screen transient UI state that outlives a single frame.
#[derive(Default)]
pub struct State {
    pub catalog: catalog::State,
    pub comparables: comparables::State,
    pub daily_filing: daily_filing::State,
    pub dashboard: dashboard::State,
    pub ratings: ratings::State,
    pub settings: settings::State,
}

/// A one-line confirmation or failure message shown above a list.
pub enum Notice {
    Info(String),
    Error(String),
}

pub fn notice(ui: &mut Ui, p: &Palette, notice: &mut Option<Notice>) {
    let Some(n) = notice else { return };
    let (text, color) = match n {
        Notice::Info(t) => (t.as_str(), p.accent),
        Notice::Error(t) => (t.as_str(), p.danger),
    };
    let mut dismiss = false;
    theme::card(p)
        .inner_margin(egui::Margin::symmetric(space::MD as i8, space::SM as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (bar, _) = ui.allocate_exact_size(egui::vec2(3.0, 18.0), Sense::hover());
                ui.painter().rect_filled(bar, 1.0, color);
                ui.add_space(space::XS);
                ui.label(text);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    dismiss = ui
                        .add(
                            Button::new(RichText::new("\u{2715}").color(p.text_secondary))
                                .frame(false),
                        )
                        .clicked();
                });
            });
        });
    ui.add_space(space::MD);
    if dismiss {
        *notice = None;
    }
}

/// A modal asking to confirm a destructive action. `Some(true)` to proceed,
/// `Some(false)` if declined (including Escape or a click outside), `None`
/// while undecided.
pub fn confirm_dialog(
    ui: &mut Ui,
    p: &Palette,
    id: &str,
    title: &str,
    body: &str,
    confirm: &str,
    cancel: &str,
) -> Option<bool> {
    let mut decision = None;
    let modal = egui::Modal::new(egui::Id::unique(id))
        .frame(theme::card(p).inner_margin(egui::Margin::same(space::XL as i8)))
        .show(ui.ctx(), |ui| {
            ui.set_max_width(420.0);
            ui.label(
                RichText::new(title)
                    .family(theme::family::serif())
                    .size(20.0),
            );
            ui.add_space(space::SM);
            ui.label(RichText::new(body).color(p.text_secondary));
            ui.add_space(space::LG);
            ui.horizontal(|ui| {
                if danger_button_filled(ui, p, confirm).clicked() {
                    decision = Some(true);
                }
                if secondary_button(ui, cancel).clicked() {
                    decision = Some(false);
                }
            });
        });
    if modal.should_close() && decision.is_none() {
        decision = Some(false);
    }
    decision
}

/// A full-height scrolling page, for screens without their own scrolling.
pub fn page(ui: &mut Ui, id: impl egui::AsIdSalt, add_contents: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink(false)
        .show(ui, add_contents);
}

/// Screen title and one-line description, with the spacing every screen shares.
pub fn screen_header(ui: &mut Ui, p: &Palette, title: &str, subtitle: &str) {
    ui.label(RichText::new(title).text_style(egui::TextStyle::Heading));
    ui.add_space(space::XS);
    ui.label(RichText::new(subtitle).color(p.text_secondary));
    ui.add_space(space::XL);
}

/// Small, letter-spaced uppercase label.
pub fn overline(ui: &mut Ui, p: &Palette, text: &str) -> Response {
    ui.label(overline_text(p, text))
}

pub fn overline_text(p: &Palette, text: &str) -> RichText {
    RichText::new(text.to_uppercase())
        .text_style(theme::text::overline())
        .extra_letter_spacing(1.4)
        .color(p.text_secondary)
}

/// The one emphasised action on a screen.
pub fn primary_button(ui: &mut Ui, p: &Palette, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).color(p.on_accent))
            .fill(p.accent)
            .stroke(egui::Stroke::NONE),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// An ordinary action.
pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    ui.button(text)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A destructive action, deliberately understated.
pub fn danger_button(ui: &mut Ui, p: &Palette, text: &str) -> Response {
    ui.add(Button::new(RichText::new(text).color(p.danger)).frame(false))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A destructive action at the point of confirmation.
pub fn danger_button_filled(ui: &mut Ui, p: &Palette, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).color(p.on_accent))
            .fill(p.danger)
            .stroke(egui::Stroke::NONE),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}
