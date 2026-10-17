//! The ticker strip: every index on the market, with its change since the
//! previous close. Scrolls when it does not fit.

use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, Ui, vec2};

use super::chart::movement_color;
use crate::format;
use crate::theme::{self, Palette, space};

const HEIGHT: f32 = 36.0;
const GAP: f32 = 36.0;
/// Scroll speed when the strip overflows, in points per second.
const SPEED: f32 = 36.0;

pub struct Quote {
    pub ticker: String,
    pub value: f64,
    pub change: Option<f64>,
}

pub fn show(ui: &mut Ui, p: &Palette, quotes: &[Quote]) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, theme::RADIUS, p.surface);
    painter.rect_stroke(
        rect,
        theme::RADIUS,
        Stroke::new(1.0, p.border),
        egui::StrokeKind::Inside,
    );

    let ticker_font = FontId::new(12.0, theme::family::sans_semibold());
    let value_font = FontId::new(13.0, theme::family::mono_medium());
    let change_font = FontId::new(12.0, egui::FontFamily::Monospace);
    let layout = |text: String, font: &FontId, color| {
        ui.fonts_mut(|f| f.layout_no_wrap(text, font.clone(), color))
    };
    let items: Vec<_> = quotes
        .iter()
        .map(|q| {
            let change = q.change.unwrap_or(0.0);
            let arrow = if change >= 0.005 {
                "\u{25B2}"
            } else if change <= -0.005 {
                "\u{25BC}"
            } else {
                "\u{25C6}"
            };
            [
                layout(q.ticker.clone(), &ticker_font, p.text_secondary),
                layout(format::index(q.value), &value_font, p.text_primary),
                layout(
                    format!("{arrow} {}", format::signed(change)),
                    &change_font,
                    movement_color(p, change),
                ),
            ]
        })
        .collect();
    let item_width = |g: &[std::sync::Arc<egui::Galley>; 3]| {
        g.iter().map(|g| g.size().x).sum::<f32>() + 2.0 * space::SM
    };
    let total: f32 = items.iter().map(item_width).sum::<f32>() + GAP * items.len() as f32;

    let inner = rect.shrink2(vec2(space::MD, 0.0));
    let overflow = total > inner.width();
    let offset = if overflow {
        ui.ctx().request_repaint();
        (ui.input(|i| i.time) as f32 * SPEED) % total
    } else {
        0.0
    };
    let clip = painter.with_clip_rect(inner);
    let passes = if overflow { 2 } else { 1 };
    for pass in 0..passes {
        let mut x = inner.left() - offset + pass as f32 * total;
        for item in &items {
            for (k, galley) in item.iter().enumerate() {
                let pos = Pos2::new(x, rect.center().y);
                clip.galley(
                    Align2::LEFT_CENTER.anchor_size(pos, galley.size()).min,
                    galley.clone(),
                    p.text_primary,
                );
                x += galley.size().x + if k < 2 { space::SM } else { 0.0 };
            }
            x += GAP;
            let sep = Rect::from_center_size(
                Pos2::new(x - GAP / 2.0, rect.center().y),
                vec2(1.0, HEIGHT * 0.4),
            );
            clip.rect_filled(sep, 0.0, p.border);
        }
    }
}
