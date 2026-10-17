//! Top-level application state and navigation.

use egui::ThemePreference;
use rusqlite::Connection;

use crate::engine::{index::Reading, market, run};
use crate::model::user::UserProfile;
use crate::settings::{Paths, Settings, ThemeChoice};
use crate::{catalog, copy, db, theme, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Comparables,
    Ratings,
    DailyFiling,
    Notices,
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
            Self::Notices => copy::nav::NOTICES,
            Self::Catalog => copy::nav::CATALOG,
            Self::Threshold => copy::nav::THRESHOLD,
            Self::Reports => copy::nav::REPORTS,
            Self::Settings => copy::nav::SETTINGS,
        }
    }
}

/// Everything a screen may need. Exists only once the database is open.
pub struct Session {
    pub conn: Connection,
    pub paths: Paths,
    pub settings: Settings,
    pub schema_version: usize,
    pub user: UserProfile,
    /// Incremented whenever the record is written, so screens know to re-query.
    pub revision: u64,
    /// The latest index reading, recomputed whenever `revision` changes.
    pub reading: Option<Reading>,
    /// The revision `reading` was computed at.
    computed_revision: Option<u64>,
    /// Notices not yet seen on the Notices screen.
    pub unread_notices: usize,
    /// The revision `unread_notices` was counted at.
    unread_revision: Option<u64>,
    /// The market day last checked.
    market_day: Option<jiff::civil::Date>,
    /// Why the last market check failed, if it did.
    market_error: Option<String>,
    /// Why the last recomputation failed, if it did.
    compute_error: Option<String>,
}

impl Session {
    fn open() -> anyhow::Result<Self> {
        let paths = Paths::discover()?;
        let settings = Settings::load(&paths.settings);
        let mut conn = db::open(&paths.database)?;
        catalog::sync(&mut conn, &catalog::seed()?)?;
        let schema_version = db::schema_version(&conn)?;
        let user = db::user::ensure(&conn, rand::random(), jiff::Timestamp::now())?;
        Ok(Self {
            conn,
            paths,
            settings,
            schema_version,
            user,
            revision: 0,
            reading: None,
            computed_revision: None,
            unread_notices: 0,
            unread_revision: None,
            market_day: None,
            market_error: None,
            compute_error: None,
        })
    }

    /// Failures from the market check and the last recomputation.
    pub fn engine_errors(&self) -> impl Iterator<Item = &str> {
        [&self.market_error, &self.compute_error]
            .into_iter()
            .filter_map(|e| e.as_deref())
    }

    /// Opens and closes the market as the calendar requires, and recomputes the
    /// index when the record has changed. Returns the time until the next
    /// local midnight, when this must run again.
    pub fn maintain(&mut self) -> std::time::Duration {
        let now = jiff::Timestamp::now();
        let tz = jiff::tz::TimeZone::system();
        let today = market::day_of(now, &tz);
        if self.market_day != Some(today) {
            self.market_day = Some(today);
            self.market_error = match run::market_check(&self.conn, now, &tz) {
                Ok(activity) => {
                    if activity.opened || !activity.closed.is_empty() {
                        self.revision += 1;
                    }
                    None
                }
                Err(e) => Some(format!("{} {e:#}", copy::engine::MARKET_FAILED)),
            };
        }
        if self.computed_revision != Some(self.revision) {
            self.computed_revision = Some(self.revision);
            self.compute_error = match run::recompute(&mut self.conn, now, &tz) {
                Ok(run::Recomputed {
                    reading,
                    adjustments,
                    introduced,
                }) => {
                    self.reading = Some(reading);
                    if introduced.is_some() {
                        // The peer set changed: recompute on the next frame.
                        self.revision += 1;
                    } else if !adjustments.is_empty() {
                        // The threshold and notices changed; screens must
                        // re-query, but the index itself is current.
                        self.revision += 1;
                        self.computed_revision = Some(self.revision);
                    }
                    None
                }
                Err(e) => Some(format!("{} {e:#}", copy::engine::FAILED)),
            };
        }
        if self.unread_revision != Some(self.revision) {
            self.unread_revision = Some(self.revision);
            self.unread_notices = db::notices::unread_count(&self.conn).unwrap_or(0);
        }
        let midnight = market::start_of(today.tomorrow().expect("date in range"), &tz);
        std::time::Duration::try_from(midnight.duration_since(now))
            .unwrap_or_default()
            .saturating_add(std::time::Duration::from_secs(1))
    }
}

pub struct Standing {
    session: Result<Session, String>,
    screen: Screen,
    ui_state: ui::State,
}

impl Standing {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let session = Session::open().map_err(|e| format!("{e:#}"));
        if let Ok(session) = &session {
            apply_theme(&cc.egui_ctx, session.settings.theme);
        }
        Self {
            session,
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

        let session = match &mut self.session {
            Ok(session) => session,
            Err(message) => {
                ui::startup_error::show(ui, &p, message);
                return;
            }
        };
        let until_midnight = session.maintain();
        ui.ctx().request_repaint_after(until_midnight);

        if !session.user.onboarding_complete {
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(p.background)
                        .inner_margin(egui::Margin::same(theme::space::XL as i8)),
                )
                .show(ui, |ui| {
                    ui::page(ui, "onboarding", |ui| {
                        ui::onboarding::show(ui, &p, session, &mut self.ui_state.onboarding)
                    })
                });
            if session.user.onboarding_complete {
                self.screen = Screen::Dashboard;
            }
            return;
        }

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
            .show(ui, |ui| ui::nav::show(ui, &p, &mut self.screen, session));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.background)
                    .inner_margin(egui::Margin::same(theme::space::XL as i8)),
            )
            .show(ui, |ui| {
                for err in session.engine_errors() {
                    ui.label(egui::RichText::new(err).color(p.danger));
                    ui.add_space(theme::space::MD);
                }
                let state = &mut self.ui_state;
                let pending = |ui: &mut egui::Ui, (title, subtitle): (&str, &str)| {
                    ui::page(ui, title, |ui| ui::pending::show(ui, &p, title, subtitle))
                };
                match self.screen {
                    Screen::Dashboard => ui::page(ui, "dashboard", |ui| {
                        ui::dashboard::show(ui, &p, session, &mut state.dashboard)
                    }),
                    Screen::Comparables => {
                        ui::comparables::show(ui, &p, session, &mut state.comparables)
                    }
                    Screen::Ratings => ui::ratings::show(ui, &p, session, &mut state.ratings),
                    Screen::DailyFiling => ui::page(ui, "daily-filing", |ui| {
                        ui::daily_filing::show(ui, &p, session, &mut state.daily_filing)
                    }),
                    Screen::Notices => ui::page(ui, "notices", |ui| {
                        ui::notices::show(ui, &p, session, &mut state.notices)
                    }),
                    Screen::Catalog => ui::catalog::show(ui, &p, session, &mut state.catalog),
                    Screen::Threshold => ui::page(ui, "threshold", |ui| {
                        ui::threshold::show(ui, &p, session, &mut state.threshold)
                    }),
                    Screen::Reports => pending(ui, copy::pending::REPORTS),
                    Screen::Settings => ui::page(ui, "settings", |ui| {
                        ui::settings::show(ui, &p, session, &mut state.settings)
                    }),
                }
            });
    }
}
