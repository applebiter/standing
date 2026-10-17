use egui::{RichText, Sense, Ui};

use crate::app::{Screen, Session};
use crate::copy;
use crate::theme::{self, Palette, space};

pub fn show(ui: &mut Ui, p: &Palette, screen: &mut Screen, session: &Session) {
    ui.add_space(space::XS);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM + 2.0;
        ui.add(egui::Image::new(&emblem(ui.ctx())).fit_to_exact_size(egui::vec2(28.0, 28.0)));
        ui.label(
            RichText::new(copy::APP_NAME.to_uppercase())
                .family(theme::family::sans_semibold())
                .size(13.0)
                .extra_letter_spacing(2.6)
                .color(p.text_primary),
        );
    });
    ui.add_space(space::LG);

    section(ui, p, copy::nav::SECTION_MARKET);
    for s in [
        Screen::Dashboard,
        Screen::Comparables,
        Screen::Ratings,
        Screen::DailyFiling,
        Screen::Notices,
    ] {
        let count = (s == Screen::Notices).then_some(session.unread_notices);
        item(ui, p, screen, s, count.filter(|n| *n > 0));
    }
    ui.add_space(space::MD);
    section(ui, p, copy::nav::SECTION_REFERENCE);
    for s in [Screen::Catalog, Screen::Threshold, Screen::Reports] {
        item(ui, p, screen, s, None);
    }
    ui.add_space(space::MD);
    section(ui, p, copy::nav::SECTION_SYSTEM);
    item(ui, p, screen, Screen::Settings, None);

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        let small = |t: String| RichText::new(t).small().color(p.text_secondary);
        ui.label(small(format!(
            "{} {}",
            copy::status::BUILD,
            env!("CARGO_PKG_VERSION")
        )));
        ui.label(small(format!(
            "{} {}",
            copy::status::SCHEMA_VERSION,
            session.schema_version
        )));
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.0, p.accent);
            ui.label(small(copy::status::DATABASE_OK.to_owned()));
        });
    });
}

fn section(ui: &mut Ui, p: &Palette, label: &str) {
    ui.add_space(space::XS);
    super::overline(ui, p, label);
    ui.add_space(space::XS);
}

fn item(ui: &mut Ui, p: &Palette, current: &mut Screen, target: Screen, count: Option<usize>) {
    let selected = *current == target;
    let height = 28.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());

    if selected {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.5),
        );
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(2.0, height));
        ui.painter().rect_filled(bar, 0.0, p.accent);
    } else if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            p.border.gamma_multiply(0.25),
        );
    }

    let font = ui
        .style()
        .text_styles
        .get(&theme::text::nav())
        .cloned()
        .unwrap_or_default();
    let font = if selected {
        egui::FontId::new(font.size, theme::family::sans_medium())
    } else {
        font
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(space::MD - 2.0, 0.0),
        egui::Align2::LEFT_CENTER,
        target.label(),
        font,
        if selected {
            p.text_primary
        } else {
            p.text_secondary
        },
    );

    if let Some(n) = count {
        let text = if n > 99 {
            "99+".to_owned()
        } else {
            n.to_string()
        };
        let galley = ui.painter().layout_no_wrap(
            text,
            egui::FontId::new(11.0, theme::family::mono_medium()),
            p.on_accent,
        );
        let pill = egui::Rect::from_center_size(
            egui::pos2(
                rect.right() - space::SM - galley.size().x / 2.0 - 6.0,
                rect.center().y,
            ),
            egui::vec2(galley.size().x + 12.0, 18.0),
        );
        ui.painter().rect_filled(pill, 9.0, p.accent);
        ui.painter()
            .galley(pill.center() - galley.size() / 2.0, galley, p.on_accent);
    }

    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        *current = target;
    }
}

/// The emblem beside the wordmark, decoded once and kept as a texture.
fn emblem(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::unique("nav-emblem");
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return texture;
    }
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icons/emblem-96.png"))
            .expect("embedded emblem is a valid PNG");
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [icon.width as usize, icon.height as usize],
        &icon.rgba,
    );
    let texture = ctx.load_texture("nav-emblem", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    texture
}
