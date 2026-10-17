//! Onboarding: profile, Dimension Review, Comparables and the Enough
//! Threshold. Progress is saved at every step, so it can be resumed.

use egui::{Align, Layout, RichText, Ui};
use jiff::Timestamp;

use super::form::{self, FieldSpec};
use crate::app::Session;
use crate::db;
use crate::model::comparable::{Comparable, ComparableDraft, Field as ComparableField};
use crate::model::dimension::Dimension;
use crate::model::user::{ProfileDraft, ProfileField};
use crate::model::{self, FieldError, FieldErrors};
use crate::theme::{self, Palette, space};
use crate::{copy, format};

const WIDTH: f32 = 640.0;
const STEPS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Profile,
    Review,
    Comparables,
    Threshold,
}

impl Step {
    fn from_index(i: u32) -> Self {
        match i {
            0 => Self::Profile,
            1 => Self::Review,
            2 => Self::Comparables,
            _ => Self::Threshold,
        }
    }

    fn index(self) -> u32 {
        self as u32
    }
}

#[derive(Default)]
pub struct State {
    profile: Option<ProfileDraft>,
    profile_errors: FieldErrors<ProfileField>,
    /// Dimensions under review, in catalog order.
    dimensions: Option<Vec<Dimension>>,
    comparables: Option<Vec<Comparable>>,
    comparable: ComparableDraft,
    comparable_errors: FieldErrors<ComparableField>,
    comparables_attempted: bool,
    threshold: String,
    threshold_error: Option<FieldError>,
    error: Option<String>,
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let step = Step::from_index(session.user.onboarding_step);
    let pad = ((ui.available_width() - WIDTH) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_width(WIDTH.min(ui.available_width()));
            ui.add_space(space::LG);
            header(ui, p, step);
            if let Some(err) = &state.error {
                ui.label(RichText::new(err).color(p.danger));
                ui.add_space(space::MD);
            }
            match step {
                Step::Profile => profile(ui, p, session, state),
                Step::Review => review(ui, p, session, state),
                Step::Comparables => comparables(ui, p, session, state),
                Step::Threshold => threshold(ui, p, session, state),
            }
            ui.add_space(space::XXL);
        });
    });
}

fn header(ui: &mut Ui, p: &Palette, step: Step) {
    ui.label(
        RichText::new(copy::APP_NAME.to_uppercase())
            .family(theme::family::sans_semibold())
            .size(13.0)
            .extra_letter_spacing(2.6),
    );
    ui.add_space(space::LG);
    // Progress through the steps.
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
    let gap = space::XS;
    let segment = (rect.width() - gap * (STEPS - 1) as f32) / STEPS as f32;
    for i in 0..STEPS {
        let x = rect.left() + i as f32 * (segment + gap);
        let r = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(segment, 4.0));
        let color = if i <= step.index() {
            p.accent
        } else {
            p.border
        };
        ui.painter().rect_filled(r, 2.0, color);
    }
    ui.add_space(space::MD);
    super::overline(ui, p, &copy::onboarding::step_of(step.index() + 1, STEPS));
    ui.add_space(space::XS);
    let (title, subtitle) = match step {
        Step::Profile => (
            copy::onboarding::PROFILE_TITLE,
            copy::onboarding::PROFILE_SUBTITLE,
        ),
        Step::Review => (
            copy::onboarding::REVIEW_TITLE,
            copy::onboarding::REVIEW_SUBTITLE,
        ),
        Step::Comparables => (
            copy::onboarding::COMPARABLES_TITLE,
            copy::onboarding::COMPARABLES_SUBTITLE,
        ),
        Step::Threshold => (
            copy::onboarding::THRESHOLD_TITLE,
            copy::onboarding::THRESHOLD_SUBTITLE,
        ),
    };
    ui.label(RichText::new(title).text_style(egui::TextStyle::Heading));
    ui.add_space(space::XS);
    ui.label(RichText::new(subtitle).color(p.text_secondary));
    ui.add_space(space::XL);
}

fn go_to(session: &mut Session, state: &mut State, step: Step, position: u32) {
    match db::user::set_onboarding_progress(&session.conn, step.index(), position) {
        Ok(()) => {
            session.user.onboarding_step = step.index();
            session.user.review_position = position;
            state.error = None;
        }
        Err(e) => state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED)),
    }
}

fn profile(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    let draft = state
        .profile
        .get_or_insert_with(|| ProfileDraft::from_profile(&session.user));
    let error = |f: ProfileField| {
        state
            .profile_errors
            .iter()
            .find(|(e, _)| *e == f)
            .map(|(_, e)| *e)
    };
    let (name_error, field_error) = (error(ProfileField::DisplayName), error(ProfileField::Field));
    let mut no_scroll = false;
    form::field(
        ui,
        p,
        FieldSpec {
            label: copy::settings::DISPLAY_NAME,
            optional: false,
            help: Some(copy::onboarding::NAME_HELP),
            error: name_error,
        },
        &mut no_scroll,
        |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut draft.display_name).desired_width(f32::INFINITY),
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
                egui::TextEdit::singleline(&mut draft.field)
                    .hint_text(copy::onboarding::FIELD_HINT)
                    .desired_width(f32::INFINITY),
            );
        },
    );
    ui.add_space(space::SM);
    if super::primary_button(ui, p, copy::onboarding::CONTINUE).clicked() {
        match draft.validate() {
            Err(errors) => state.profile_errors = errors,
            Ok((name, field)) => {
                state.profile_errors.clear();
                match db::user::update(&session.conn, &name, field.as_deref()) {
                    Ok(()) => {
                        session.user.display_name = name;
                        session.user.field = field;
                        let position = session.user.review_position;
                        go_to(session, state, Step::Review, position);
                    }
                    Err(e) => {
                        state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED))
                    }
                }
            }
        }
    }
}

fn review(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    if state.dimensions.is_none() {
        match db::dimensions::visible(&session.conn) {
            Ok(d) => state.dimensions = Some(d),
            Err(e) => {
                state.error = Some(format!("{} {e:#}", copy::onboarding::LOAD_FAILED));
                return;
            }
        }
    }
    let dims = state.dimensions.as_mut().expect("loaded");
    let total = dims.len() as u32;
    let position = session.user.review_position.min(total);
    if total == 0 || position >= total {
        finish_review(session, state);
        return;
    }
    let i = position as usize;

    let mut toggle = false;
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let d = &dims[i];
        ui.horizontal(|ui| {
            super::overline(ui, p, &format!("{} \u{b7} {}", d.sector, d.industry));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(&d.id).monospace().color(p.text_secondary));
            });
        });
        ui.add_space(space::SM);
        ui.label(
            RichText::new(&d.name)
                .family(theme::family::serif())
                .size(28.0),
        );
        ui.add_space(space::XS);
        ui.label(RichText::new(&d.definition).color(p.text_secondary));
        ui.add_space(space::LG);
        let mut applies = d.applies;
        if ui
            .checkbox(
                &mut applies,
                RichText::new(copy::onboarding::APPLIES).size(16.0),
            )
            .changed()
        {
            toggle = true;
        }
    });

    let no_focus = ui.memory(|m| m.focused().is_none());
    let (key_next, key_previous, key_toggle) = ui.input(|input| {
        (
            no_focus
                && (input.key_pressed(egui::Key::ArrowRight)
                    || input.key_pressed(egui::Key::Enter)),
            no_focus && input.key_pressed(egui::Key::ArrowLeft),
            no_focus && input.key_pressed(egui::Key::Space),
        )
    });
    if toggle || key_toggle {
        let d = &mut dims[i];
        match db::dimensions::set_applies(&session.conn, &d.id, !d.applies) {
            Ok(()) => d.applies = !d.applies,
            Err(e) => state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED)),
        }
    }

    ui.add_space(space::MD);
    ui.label(
        RichText::new(copy::onboarding::reviewed(position, total))
            .family(theme::family::mono_medium())
            .color(p.text_secondary),
    );
    ui.add(
        egui::ProgressBar::new(position as f32 / total as f32)
            .desired_height(4.0)
            .fill(p.accent),
    );
    ui.add_space(space::MD);

    let mut next = key_next;
    let mut previous = key_previous && position > 0;
    let mut back = false;
    ui.horizontal(|ui| {
        if super::primary_button(ui, p, copy::onboarding::NEXT).clicked() {
            next = true;
        }
        if ui
            .add_enabled(position > 0, egui::Button::new(copy::onboarding::PREVIOUS))
            .clicked()
        {
            previous = true;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if super::secondary_button(ui, copy::onboarding::BACK).clicked() {
                back = true;
            }
        });
    });
    ui.add_space(space::SM);
    ui.label(
        RichText::new(copy::onboarding::KEYS)
            .small()
            .color(p.text_secondary),
    );
    ui.add_space(space::LG);
    let skip = ui
        .add(
            egui::Button::new(
                RichText::new(copy::onboarding::SKIP_REMAINING)
                    .small()
                    .color(p.text_secondary),
            )
            .frame(false),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked();

    if back {
        go_to(session, state, Step::Profile, position);
    } else if skip {
        go_to(session, state, Step::Review, total);
        finish_review(session, state);
    } else if next {
        go_to(session, state, Step::Review, position + 1);
        if position + 1 >= total {
            finish_review(session, state);
        }
    } else if previous {
        go_to(session, state, Step::Review, position - 1);
    }
}

/// Leaves the Dimension Review. Applicability changed, so screens re-query.
fn finish_review(session: &mut Session, state: &mut State) {
    let position = session.user.review_position;
    go_to(session, state, Step::Comparables, position);
    session.revision += 1;
}

fn comparables(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    if state.comparables.is_none() {
        match db::comparables::active(&session.conn) {
            Ok(c) => state.comparables = Some(c),
            Err(e) => {
                state.error = Some(format!("{} {e:#}", copy::onboarding::LOAD_FAILED));
                return;
            }
        }
    }
    let error = |f: ComparableField| {
        if !state.comparables_attempted {
            return None;
        }
        state
            .comparable_errors
            .iter()
            .find(|(e, _)| *e == f)
            .map(|(_, e)| *e)
    };
    let (name_error, rel_error) = (
        error(ComparableField::Name),
        error(ComparableField::Relationship),
    );
    let mut no_scroll = false;

    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        form::field(
            ui,
            p,
            FieldSpec {
                label: copy::comparable_form::NAME,
                optional: false,
                help: Some(copy::comparable_form::NAME_HELP),
                error: name_error,
            },
            &mut no_scroll,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.comparable.name)
                        .desired_width(f32::INFINITY),
                );
            },
        );
        form::field(
            ui,
            p,
            FieldSpec {
                label: copy::comparable_form::RELATIONSHIP,
                optional: true,
                help: None,
                error: rel_error,
            },
            &mut no_scroll,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.comparable.relationship)
                        .hint_text(copy::comparable_form::RELATIONSHIP_HINT)
                        .desired_width(f32::INFINITY),
                );
            },
        );
        if super::secondary_button(ui, copy::onboarding::ADD_COMPARABLE).clicked() {
            state.comparables_attempted = true;
            match state.comparable.validate() {
                Err(errors) => state.comparable_errors = errors,
                Ok(fields) => {
                    state.comparable_errors.clear();
                    match db::comparables::insert(&session.conn, &fields, Timestamp::now()) {
                        Ok(c) => {
                            state.comparables.get_or_insert_with(Vec::new).push(c);
                            state.comparable = ComparableDraft::default();
                            state.comparables_attempted = false;
                            session.revision += 1;
                        }
                        Err(e) => {
                            state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED))
                        }
                    }
                }
            }
        }
    });

    let added = state.comparables.clone().unwrap_or_default();
    ui.add_space(space::MD);
    super::overline(ui, p, &copy::onboarding::peer_set(added.len()));
    ui.add_space(space::SM);
    if added.is_empty() {
        ui.label(RichText::new(copy::onboarding::NONE_YET).color(p.text_secondary));
    }
    let mut remove = None;
    for c in &added {
        ui.horizontal(|ui| {
            ui.add_sized(
                [72.0, 20.0],
                egui::Label::new(RichText::new(&c.ticker).family(theme::family::mono_medium())),
            );
            ui.label(&c.name);
            if let Some(rel) = &c.relationship {
                ui.label(RichText::new(rel).color(p.text_secondary));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if super::danger_button(ui, p, copy::onboarding::REMOVE).clicked() {
                    remove = Some(c.id);
                }
            });
        });
    }
    if let Some(id) = remove {
        match db::comparables::delete(&mut session.conn, id) {
            Ok(()) => {
                if let Some(list) = &mut state.comparables {
                    list.retain(|c| c.id != id);
                }
                session.revision += 1;
            }
            Err(e) => state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED)),
        }
    }

    ui.add_space(space::LG);
    ui.horizontal(|ui| {
        let ready = !added.is_empty();
        if ui
            .add_enabled_ui(ready, |ui| {
                super::primary_button(ui, p, copy::onboarding::CONTINUE)
            })
            .inner
            .clicked()
        {
            let position = session.user.review_position;
            go_to(session, state, Step::Threshold, position);
        }
        if !ready {
            ui.label(
                RichText::new(copy::onboarding::NEED_ONE)
                    .small()
                    .color(p.text_secondary),
            );
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if super::secondary_button(ui, copy::onboarding::BACK).clicked() {
                let position = session.user.review_position;
                go_to(session, state, Step::Review, position.saturating_sub(1));
            }
        });
    });
}

fn threshold(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut State) {
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if let Some(r) = &session.reading {
            ui.label(copy::onboarding::psi_now(&format::index(r.psi)));
            ui.add_space(space::MD);
        }
        super::overline(ui, p, copy::onboarding::THRESHOLD_FIELD);
        ui.add_space(space::XS);
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.threshold)
                .desired_width(160.0)
                .font(egui::TextStyle::Monospace),
        );
        if response.changed() {
            state.threshold_error = None;
        }
        match state.threshold_error {
            Some(e) => {
                ui.label(RichText::new(copy::form::error(e)).small().color(p.danger));
            }
            None => {
                ui.label(
                    RichText::new(copy::threshold::FIELD_HELP)
                        .small()
                        .color(p.text_secondary),
                );
            }
        }
    });
    ui.add_space(space::LG);
    ui.horizontal(|ui| {
        if super::primary_button(ui, p, copy::onboarding::FINISH).clicked() {
            match model::threshold(&state.threshold) {
                Err(e) => state.threshold_error = Some(e),
                Ok(value) => finish(session, state, value),
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if super::secondary_button(ui, copy::onboarding::BACK).clicked() {
                let position = session.user.review_position;
                go_to(session, state, Step::Comparables, position);
            }
        });
    });
}

/// Sets the initial threshold and completes onboarding. Recomputation
/// follows, and applies the threshold at once.
fn finish(session: &mut Session, state: &mut State, value: f64) {
    let result = (|| -> db::Result<()> {
        let tx = session.conn.unchecked_transaction()?;
        db::threshold::set(&tx, value, db::threshold::Reason::Initial, Timestamp::now())?;
        db::user::complete_onboarding(&tx)?;
        tx.commit()?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            session.user.onboarding_complete = true;
            session.revision += 1;
            *state = State::default();
        }
        Err(e) => state.error = Some(format!("{} {e:#}", copy::onboarding::SAVE_FAILED)),
    }
}
