use egui::{RichText, Ui};

use crate::copy;
use crate::theme::{self, Palette};

/// A screen whose module is not yet installed.
pub fn show(ui: &mut Ui, p: &Palette, title: &str, subtitle: &str) {
    super::screen_header(ui, p, title, subtitle);
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(copy::pending::BODY).color(p.text_secondary));
    });
}
