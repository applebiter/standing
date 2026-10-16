//! Top-level application state and navigation.

use egui::ThemePreference;
use rusqlite::Connection;

use crate::settings::{Paths, Settings, ThemeChoice};
use crate::{copy, db, theme, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Comparables,
    Ratings,
    DailyFiling,
    Catalog,
    Threshold,
    Reports,
    Settings,
}

impl Screen {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => copy::nav::DASHBOARD,
            Self::Comparables => copy::nav::COMPARABLES,
            Self::Ratings => copy::nav::RATINGS,
            Self::DailyFiling => copy::nav::DAILY_FILING,
            Self::Catalog => copy::nav::CATALOG,
            Self::Threshold => copy::nav::THRESHOLD,
            Self::Reports => copy::nav::REPORTS,
            Self::Settings => copy::nav::SETTINGS,
        }
    }
}

/// Everything a screen may need. Exists only once the database is open.
pub struct Profile {
    pub conn: Connection,
    pub paths: Paths,
    pub settings: Settings,
    pub schema_version: usize,
}

impl Profile {
    fn open() -> anyhow::Result<Self> {
        let paths = Paths::discover()?;
        let settings = Settings::load(&paths.settings);
        let conn = db::open(&paths.database)?;
        let schema_version = db::schema_version(&conn)?;
        Ok(Self {
            conn,
            paths,
            settings,
            schema_version,
        })
    }
}

pub struct Standing {
    profile: Result<Profile, String>,
    screen: Screen,
    ui_state: ui::State,
}

impl Standing {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let profile = Profile::open().map_err(|e| format!("{e:#}"));
        if let Ok(profile) = &profile {
            apply_theme(&cc.egui_ctx, profile.settings.theme);
        }
        Self {
            profile,
            screen: Screen::Dashboard,
            ui_state: ui::State::default(),
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, choice: ThemeChoice) {
    ctx.set_theme(match choice {
        ThemeChoice::System => ThemePreference::System,
        ThemeChoice::Light => ThemePreference::Light,
        ThemeChoice::Dark => ThemePreference::Dark,
    });
}

impl eframe::App for Standing {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let p = *theme::palette(ui.visuals().dark_mode);

        let profile = match &mut self.profile {
            Ok(profile) => profile,
            Err(message) => {
                ui::startup_error::show(ui, &p, message);
                return;
            }
        };

        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(theme::SIDEBAR_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .inner_margin(egui::Margin {
                        left: theme::space::MD as i8,
                        right: theme::space::MD as i8,
                        top: theme::space::LG as i8,
                        bottom: theme::space::MD as i8,
                    }),
            )
            .show(ui, |ui| ui::nav::show(ui, &p, &mut self.screen, profile));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.background)
                    .inner_margin(egui::Margin::same(theme::space::XL as i8)),
            )
            .show(ui, |ui| {
                let state = &mut self.ui_state;
                let pending = |ui: &mut egui::Ui, (title, subtitle): (&str, &str)| {
                    ui::page(ui, title, |ui| ui::pending::show(ui, &p, title, subtitle))
                };
                match self.screen {
                    Screen::Dashboard => pending(ui, copy::pending::DASHBOARD),
                    Screen::Comparables => pending(ui, copy::pending::COMPARABLES),
                    Screen::Ratings => pending(ui, copy::pending::RATINGS),
                    Screen::DailyFiling => pending(ui, copy::pending::DAILY_FILING),
                    Screen::Catalog => pending(ui, copy::pending::CATALOG),
                    Screen::Threshold => pending(ui, copy::pending::THRESHOLD),
                    Screen::Reports => pending(ui, copy::pending::REPORTS),
                    Screen::Settings => ui::page(ui, "settings", |ui| {
                        ui::settings::show(ui, &p, profile, &mut state.settings)
                    }),
                }
            });
    }
}
