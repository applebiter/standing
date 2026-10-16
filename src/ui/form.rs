//! Building blocks shared by the full-page forms.

use egui::{Align, Layout, RichText, Ui};

use crate::copy;
use crate::model::FieldError;
use crate::theme::{self, Palette, space};

/// Distinguishes one opening of a form from the next, so each starts scrolled to the top.
pub fn next_instance() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Title block. `identifier` is shown above the title when amending an existing record.
pub fn header(ui: &mut Ui, p: &Palette, identifier: Option<&str>, title: &str, subtitle: &str) {
    if let Some(id) = identifier {
        ui.label(
            RichText::new(id)
                .family(theme::family::mono_medium())
                .size(12.0)
                .color(p.text_secondary),
        );
        ui.add_space(space::XS);
    }
    ui.label(RichText::new(title).text_style(egui::TextStyle::Heading));
    ui.add_space(space::XS);
    ui.label(RichText::new(subtitle).color(p.text_secondary));
    ui.add_space(space::LG);
}

pub struct FieldSpec<'a> {
    pub label: &'a str,
    pub optional: bool,
    pub help: Option<&'a str>,
    /// Shown, with a red outline on the control, when set.
    pub error: Option<FieldError>,
}

/// A labelled form control with optional help text and error message.
/// If `scroll_to_error` is set and this field is in error, it scrolls into
/// view and clears the flag, so the first error in form order wins.
pub fn field(
    ui: &mut Ui,
    p: &Palette,
    spec: FieldSpec<'_>,
    scroll_to_error: &mut bool,
    control: impl FnOnce(&mut Ui),
) {
    // Columns lay out justified; fields should not inherit that.
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let label_row = ui.horizontal(|ui| {
            ui.spacing_mut().interact_size.y = 0.0;
            ui.label(RichText::new(spec.label).family(theme::family::sans_medium()));
            if spec.optional {
                ui.label(
                    RichText::new(copy::form::OPTIONAL)
                        .small()
                        .color(p.text_secondary),
                );
            }
        });
        if spec.error.is_some() && *scroll_to_error {
            *scroll_to_error = false;
            label_row.response.scroll_to_me(Some(Align::Center));
        }
        ui.add_space(2.0);
        ui.scope(|ui| {
            if spec.error.is_some() {
                let w = &mut ui.visuals_mut().widgets;
                for state in [&mut w.inactive, &mut w.hovered, &mut w.noninteractive] {
                    state.bg_stroke = egui::Stroke::new(1.0, p.danger);
                }
            }
            control(ui);
        });
        if let Some(text) = spec.help {
            help(ui, p, text);
        }
        if let Some(e) = spec.error {
            ui.label(RichText::new(copy::form::error(e)).small().color(p.danger));
        }
    });
    ui.add_space(space::MD);
}

pub fn help(ui: &mut Ui, p: &Palette, text: &str) {
    ui.label(RichText::new(text).small().color(p.text_secondary));
}
