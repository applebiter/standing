use egui::{RichText, Ui};

use super::form::{self, FieldSpec};
use crate::app::{self, Session};
use crate::model::FieldErrors;
use crate::model::user::{ProfileDraft, ProfileField};
use crate::settings::{Location, ThemeChoice};
use crate::theme::{self, Palette, space};
use crate::{copy, db};

/// All settings cards share one width, wide enough for the widest row.
const CARD_WIDTH: f32 = 660.0;

#[derive(Default)]
pub struct State {
    save_error: Option<String>,
    profile: Option<ProfileForm>,
}

struct ProfileForm {
    draft: ProfileDraft,
    errors: FieldErrors<ProfileField>,
    saved: bool,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    super::screen_header(ui, p, copy::settings::TITLE, copy::settings::SUBTITLE);

    profile_card(ui, p, session, state);
    ui.add_space(space::MD);

    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width().min(CARD_WIDTH));
        super::overline(ui, p, copy::settings::APPEARANCE);
        ui.add_space(space::SM);
        ui.horizontal(|ui| {
            ui.label(copy::settings::THEME);
            ui.add_space(space::MD);
            let before = session.settings.theme;
            for (choice, label) in [
                (ThemeChoice::System, copy::settings::THEME_SYSTEM),
                (ThemeChoice::Light, copy::settings::THEME_LIGHT),
                (ThemeChoice::Dark, copy::settings::THEME_DARK),
            ] {
                ui.selectable_value(&mut session.settings.theme, choice, label);
            }
            if session.settings.theme != before {
                app::apply_theme(ui.ctx(), session.settings.theme);
                state.save_error = session
                    .settings
                    .save(&session.paths.settings)
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
                RichText::new(session.paths.data_dir().display().to_string())
                    .monospace()
                    .color(p.text_primary),
            )
            .wrap(),
        );
        ui.add_space(space::XS);
        ui.label(
            RichText::new(match session.paths.location {
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

fn profile_card(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let pf = state.profile.get_or_insert_with(|| ProfileForm {
        draft: ProfileDraft::from_profile(&session.user),
        errors: Vec::new(),
        saved: false,
    });
    let error = |f: ProfileField| pf.errors.iter().find(|(e, _)| *e == f).map(|(_, e)| *e);
    let (name_error, field_error) = (error(ProfileField::DisplayName), error(ProfileField::Field));
    let mut no_scroll = false;

    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width().min(CARD_WIDTH));
        super::overline(ui, p, copy::settings::PROFILE);
        ui.add_space(space::SM);
        let before = pf.draft.clone();
        form::field(
            ui,
            p,
            FieldSpec {
                label: copy::settings::DISPLAY_NAME,
                optional: false,
                help: None,
                error: name_error,
            },
            &mut no_scroll,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut pf.draft.display_name)
                        .desired_width(f32::INFINITY),
                );
            },
        );
        form::field(
            ui,
            p,
            FieldSpec {
                label: copy::settings::FIELD,
                optional: true,
                help: Some(copy::settings::FIELD_HELP),
                error: field_error,
            },
            &mut no_scroll,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut pf.draft.field).desired_width(f32::INFINITY),
                );
            },
        );
        if pf.draft != before {
            pf.saved = false;
        }
        ui.horizontal(|ui| {
            let unchanged = pf.draft == ProfileDraft::from_profile(&session.user);
            if ui
                .add_enabled_ui(!unchanged, |ui| {
                    super::primary_button(ui, p, copy::settings::SAVE_PROFILE)
                })
                .inner
                .clicked()
            {
                match pf.draft.validate() {
                    Err(errors) => pf.errors = errors,
                    Ok((name, field)) => {
                        pf.errors.clear();
                        match db::user::update(&session.conn, &name, field.as_deref()) {
                            Ok(()) => {
                                session.user.display_name = name;
                                session.user.field = field;
                                session.revision += 1;
                                pf.draft = ProfileDraft::from_profile(&session.user);
                                pf.saved = true;
                                state.save_error = None;
                            }
                            Err(e) => {
                                state.save_error =
                                    Some(format!("{} {e:#}", copy::settings::SAVE_FAILED))
                            }
                        }
                    }
                }
            }
            if pf.saved {
                ui.label(
                    RichText::new(copy::settings::PROFILE_SAVED)
                        .small()
                        .color(p.text_secondary),
                );
            }
        });
    });
}
