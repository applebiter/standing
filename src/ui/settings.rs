use egui::{RichText, Ui};

use crate::app::{self, Profile};
use crate::copy;
use crate::settings::{Location, ThemeChoice};
use crate::theme::{self, Palette, space};

/// All settings cards share one width, wide enough for the widest row.
const CARD_WIDTH: f32 = 660.0;

#[derive(Default)]
pub struct State {
    save_error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, profile: &mut Profile, state: &mut State) {
    super::screen_header(ui, p, copy::settings::TITLE, copy::settings::SUBTITLE);

    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width().min(CARD_WIDTH));
        super::overline(ui, p, copy::settings::APPEARANCE);
        ui.add_space(space::SM);
        ui.horizontal(|ui| {
            ui.label(copy::settings::THEME);
            ui.add_space(space::MD);
            let before = profile.settings.theme;
            for (choice, label) in [
                (ThemeChoice::System, copy::settings::THEME_SYSTEM),
                (ThemeChoice::Light, copy::settings::THEME_LIGHT),
                (ThemeChoice::Dark, copy::settings::THEME_DARK),
            ] {
                ui.selectable_value(&mut profile.settings.theme, choice, label);
            }
            if profile.settings.theme != before {
                app::apply_theme(ui.ctx(), profile.settings.theme);
                state.save_error = profile
                    .settings
                    .save(&profile.paths.settings)
                    .err()
                    .map(|e| format!("{} {e:#}", copy::settings::SAVE_FAILED));
            }
        });
    });

    ui.add_space(space::MD);
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width().min(CARD_WIDTH));
        super::overline(ui, p, copy::settings::STORAGE);
        ui.add_space(space::SM);
        ui.label(copy::settings::STORAGE_LOCATION);
        ui.add_space(space::XS);
        ui.add(
            egui::Label::new(
                RichText::new(profile.paths.data_dir().display().to_string())
                    .monospace()
                    .color(p.text_primary),
            )
            .wrap(),
        );
        ui.add_space(space::XS);
        ui.label(
            RichText::new(match profile.paths.location {
                Location::Override => copy::settings::STORAGE_OVERRIDE,
                Location::Portable => copy::settings::STORAGE_PORTABLE,
                Location::System => copy::settings::STORAGE_SYSTEM,
            })
            .small()
            .color(p.text_secondary),
        );
    });

    if let Some(err) = &state.save_error {
        ui.add_space(space::MD);
        ui.label(RichText::new(err).color(p.danger));
    }
}
