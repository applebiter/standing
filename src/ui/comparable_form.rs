//! Add a Comparable, and amend an existing one.

use egui::{Align, Layout, RichText, Ui};
use jiff::Timestamp;

use super::form::{self, FieldSpec};
use crate::app::Session;
use crate::db::comparables;
use crate::model::FieldError;
use crate::model::comparable::{Comparable, ComparableDraft, Field};
use crate::theme::{Palette, space};
use crate::{copy, format};

const FORM_MAX_WIDTH: f32 = 640.0;

pub struct FormState {
    instance: u64,
    /// The Comparable being amended, or `None` when adding one.
    editing: Option<Comparable>,
    draft: ComparableDraft,
    errors: Vec<(Field, FieldError)>,
    attempted: bool,
    scroll_to_error: bool,
    write_error: bool,
}

pub enum Outcome {
    Open,
    Cancelled,
    Saved(String),
    DeleteRequested,
}

impl FormState {
    pub fn new() -> Self {
        Self::with(None, ComparableDraft::default())
    }

    pub fn edit(comparable: Comparable) -> Self {
        let draft = ComparableDraft::from_comparable(&comparable);
        Self::with(Some(comparable), draft)
    }

    fn with(editing: Option<Comparable>, draft: ComparableDraft) -> Self {
        Self {
            instance: form::next_instance(),
            editing,
            draft,
            errors: Vec::new(),
            attempted: false,
            scroll_to_error: false,
            write_error: false,
        }
    }

    pub fn editing(&self) -> Option<&Comparable> {
        self.editing.as_ref()
    }

    fn error(&self, field: Field) -> Option<FieldError> {
        if !self.attempted {
            return None;
        }
        self.errors
            .iter()
            .find(|(f, _)| *f == field)
            .map(|(_, e)| *e)
    }
}

pub fn show(ui: &mut Ui, p: &Palette, session: &mut Session, state: &mut FormState) -> Outcome {
    let mut outcome = Outcome::Open;

    super::page(ui, ("comparable-form", state.instance), |ui| {
        ui.set_max_width(ui.available_width().min(FORM_MAX_WIDTH));
        match &state.editing {
            Some(c) => {
                let added = format::local_date(&c.created_at)
                    .map(format::date_long)
                    .unwrap_or_default();
                form::header(
                    ui,
                    p,
                    Some(&c.ticker),
                    copy::comparable_form::EDIT_TITLE,
                    &copy::comparable_form::edit_subtitle(&added),
                );
            }
            None => form::header(
                ui,
                p,
                None,
                copy::comparable_form::NEW_TITLE,
                copy::comparable_form::NEW_SUBTITLE,
            ),
        }

        field(
            ui,
            p,
            state,
            Field::Name,
            copy::comparable_form::NAME,
            false,
            Some(copy::comparable_form::NAME_HELP),
            |ui, d| {
                ui.add(egui::TextEdit::singleline(&mut d.name).desired_width(f32::INFINITY));
            },
        );
        field(
            ui,
            p,
            state,
            Field::Relationship,
            copy::comparable_form::RELATIONSHIP,
            true,
            None,
            |ui, d| {
                ui.add(
                    egui::TextEdit::singleline(&mut d.relationship)
                        .hint_text(copy::comparable_form::RELATIONSHIP_HINT)
                        .desired_width(f32::INFINITY),
                );
            },
        );

        if state.attempted {
            state.errors = state.draft.validate().err().unwrap_or_default();
        }

        ui.add_space(space::LG);
        if state.attempted && !state.errors.is_empty() {
            ui.label(RichText::new(copy::comparable_form::INCOMPLETE).color(p.danger));
            ui.add_space(space::SM);
        }
        if state.write_error {
            ui.label(RichText::new(copy::comparables::WRITE_FAILED).color(p.danger));
            ui.add_space(space::SM);
        }

        ui.horizontal(|ui| {
            let submit = if state.editing.is_some() {
                copy::comparable_form::SUBMIT_EDIT
            } else {
                copy::comparable_form::SUBMIT_NEW
            };
            if super::primary_button(ui, p, submit).clicked() {
                state.attempted = true;
                match submit_form(session, state) {
                    Some(notice) => outcome = Outcome::Saved(notice),
                    None => state.scroll_to_error = !state.errors.is_empty(),
                }
            }
            if super::secondary_button(ui, copy::form::CANCEL).clicked() {
                outcome = Outcome::Cancelled;
            }
            if let Some(c) = &state.editing {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if super::danger_button(ui, p, copy::comparable_form::DELETE).clicked() {
                        outcome = Outcome::DeleteRequested;
                    }
                    let (label, active) = if c.active {
                        (copy::comparable_form::REMOVE, false)
                    } else {
                        (copy::comparable_form::RESTORE, true)
                    };
                    if super::secondary_button(ui, label).clicked() {
                        let result = comparables::set_active(&session.conn, c.id, active);
                        state.write_error = result.is_err();
                        if result.is_ok() {
                            session.revision += 1;
                            outcome = Outcome::Saved(if active {
                                copy::comparables::restored(&c.name)
                            } else {
                                copy::comparables::removed(&c.name)
                            });
                        }
                    }
                });
            }
        });
        ui.add_space(space::XXL);
    });

    outcome
}

/// Validates and writes the form. Returns the notice to show on success.
fn submit_form(session: &mut Session, state: &mut FormState) -> Option<String> {
    let fields = match state.draft.validate() {
        Ok(fields) => fields,
        Err(errors) => {
            state.errors = errors;
            return None;
        }
    };
    state.errors.clear();
    let result = match &state.editing {
        Some(c) => comparables::update(&session.conn, c.id, &fields)
            .map(|_| copy::comparables::amended(&fields.name)),
        None => comparables::insert(&session.conn, &fields, Timestamp::now())
            .map(|c| copy::comparables::added(&c.name, &c.ticker)),
    };
    state.write_error = result.is_err();
    if result.is_ok() {
        session.revision += 1;
    }
    result.ok()
}

#[allow(clippy::too_many_arguments)]
fn field(
    ui: &mut Ui,
    p: &Palette,
    state: &mut FormState,
    id: Field,
    label: &str,
    optional: bool,
    help: Option<&str>,
    control: impl FnOnce(&mut Ui, &mut ComparableDraft),
) {
    let error = state.error(id);
    let FormState {
        draft,
        scroll_to_error,
        ..
    } = state;
    form::field(
        ui,
        p,
        FieldSpec {
            label,
            optional,
            help,
            error,
        },
        scroll_to_error,
        |ui| control(ui, draft),
    );
}
