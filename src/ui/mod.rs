//! Screens and shared widgets. The only module tree that depends on egui
//! besides `app` and `theme`.

pub mod catalog;
pub mod nav;
pub mod pending;
pub mod settings;
pub mod startup_error;

use egui::{Response, RichText, Ui};

use crate::theme::{self, Palette, space};

/// Per-screen transient UI state that outlives a single frame.
#[derive(Default)]
pub struct State {
    pub catalog: catalog::State,
    pub settings: settings::State,
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
