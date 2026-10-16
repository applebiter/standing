use egui::{RichText, Ui};

use crate::copy;
use crate::theme::{Palette, space};

pub fn show(ui: &mut Ui, p: &Palette, message: &str) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.background)
                .inner_margin(egui::Margin::same(space::XXL as i8)),
        )
        .show(ui, |ui| {
            ui.label(RichText::new(copy::startup::TITLE).text_style(egui::TextStyle::Heading));
            ui.add_space(space::MD);
            ui.label(RichText::new(copy::startup::BODY).color(p.text_secondary));
            ui.add_space(space::LG);
            ui.label(RichText::new(message).monospace().color(p.danger));
        });
}
