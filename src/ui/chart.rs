//! Daily candlestick chart with an optional threshold line.

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Ui, vec2};

use crate::engine::market::Candle;
use crate::theme::{self, Palette};
use crate::{copy, format};

const AXIS_WIDTH: f32 = 64.0;
const DATE_HEIGHT: f32 = 22.0;
const TOP_PAD: f32 = 18.0;

/// Draws `candles` (one per day, oldest first) across the available width.
pub fn candlesticks(
    ui: &mut Ui,
    p: &Palette,
    candles: &[Candle],
    threshold: Option<f64>,
    height: f32,
) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    if candles.is_empty() || !ui.is_rect_visible(rect) {
        return;
    }
    let plot = Rect::from_min_max(
        rect.min + vec2(0.0, TOP_PAD),
        rect.max - vec2(AXIS_WIDTH, DATE_HEIGHT),
    );
    let painter = ui.painter_at(rect);
    let small = FontId::new(11.0, egui::FontFamily::Monospace);

    // Vertical range: every candle, and the threshold.
    let mut lo = candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let mut hi = candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    if let Some(t) = threshold {
        lo = lo.min(t);
        hi = hi.max(t);
    }
    if hi - lo < 1.0 {
        let mid = (hi + lo) / 2.0;
        (lo, hi) = (mid - 0.5, mid + 0.5);
    }
    let pad = (hi - lo) * 0.08;
    (lo, hi) = (lo - pad, hi + pad);
    let y_of = |v: f64| plot.bottom() - ((v - lo) / (hi - lo)) as f32 * plot.height();

    // Gridlines at a round step.
    let step = nice_step((hi - lo) / 4.0);
    let decimals = if step < 1.0 { 2 } else { 0 };
    let mut v = (lo / step).ceil() * step;
    while v <= hi {
        let y = y_of(v);
        painter.hline(
            plot.x_range(),
            y,
            Stroke::new(1.0, p.border.gamma_multiply(0.6)),
        );
        painter.text(
            Pos2::new(plot.right() + 10.0, y),
            Align2::LEFT_CENTER,
            format!("{v:.decimals$}"),
            small.clone(),
            p.text_secondary,
        );
        v += step;
    }

    // Candles.
    let slot = plot.width() / candles.len() as f32;
    let body = (slot * 0.6).clamp(2.0, 14.0);
    let x_of = |i: usize| plot.left() + slot * (i as f32 + 0.5);
    for (i, c) in candles.iter().enumerate() {
        let color = movement_color(p, c.close - c.open);
        let x = x_of(i);
        painter.vline(x, y_of(c.high)..=y_of(c.low), Stroke::new(1.0, color));
        let (top, bottom) = (y_of(c.open.max(c.close)), y_of(c.open.min(c.close)));
        let body_rect = Rect::from_min_max(
            Pos2::new(x - body / 2.0, top),
            Pos2::new(x + body / 2.0, bottom.max(top + 1.0)),
        );
        painter.rect_filled(body_rect, 1.0, color);
    }

    // The threshold.
    if let Some(t) = threshold {
        let y = y_of(t);
        painter.extend(egui::Shape::dashed_line(
            &[Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            Stroke::new(1.5, p.accent),
            6.0,
            4.0,
        ));
        painter.text(
            Pos2::new(plot.left() + 4.0, y - 4.0),
            Align2::LEFT_BOTTOM,
            copy::dashboard::THRESHOLD_LINE,
            FontId::new(11.0, theme::family::sans_medium()),
            p.accent,
        );
        painter.text(
            Pos2::new(plot.right() + 10.0, y),
            Align2::LEFT_CENTER,
            format::index(t),
            FontId::new(11.0, theme::family::mono_medium()),
            p.accent,
        );
    }

    // Dates: first, middle and last.
    let mut labelled = vec![0, candles.len() / 2, candles.len() - 1];
    labelled.dedup();
    for i in labelled {
        let align = match i {
            0 => Align2::LEFT_TOP,
            i if i == candles.len() - 1 => Align2::RIGHT_TOP,
            _ => Align2::CENTER_TOP,
        };
        let x = match align {
            Align2::LEFT_TOP => plot.left(),
            Align2::RIGHT_TOP => plot.right(),
            _ => x_of(i),
        };
        painter.text(
            Pos2::new(x, plot.bottom() + 6.0),
            align,
            format::date_day_month(candles[i].day),
            small.clone(),
            p.text_secondary,
        );
    }

    // Hover readout.
    if let Some(pos) = response
        .hover_pos()
        .filter(|pos| plot.x_range().contains(pos.x))
    {
        let i = (((pos.x - plot.left()) / slot) as usize).min(candles.len() - 1);
        painter.vline(
            x_of(i),
            plot.y_range(),
            Stroke::new(1.0, p.text_secondary.gamma_multiply(0.4)),
        );
        let c = candles[i];
        response.on_hover_ui_at_pointer(|ui| {
            ui.spacing_mut().interact_size.y = 0.0;
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.set_min_width(150.0);
            ui.add(
                egui::Label::new(egui::RichText::new(format::date_long(c.day)).strong()).extend(),
            );
            ui.add_space(4.0);
            egui::Grid::new("candle-readout")
                .num_columns(2)
                .show(ui, |ui| {
                    for (label, value) in [
                        (copy::dashboard::OPEN, c.open),
                        (copy::dashboard::HIGH, c.high),
                        (copy::dashboard::LOW, c.low),
                        (copy::dashboard::CLOSE, c.close),
                    ] {
                        ui.label(egui::RichText::new(label).color(p.text_secondary));
                        ui.label(egui::RichText::new(format::index(value)).monospace());
                        ui.end_row();
                    }
                });
        });
    }
}

/// Gain, loss, or neutral, for a change in index value.
pub fn movement_color(p: &Palette, change: f64) -> Color32 {
    if change >= 0.005 {
        p.gain
    } else if change <= -0.005 {
        p.loss
    } else {
        p.text_secondary
    }
}

/// A round step near `raw`: 1, 2, 2.5 or 5 times a power of ten.
fn nice_step(raw: f64) -> f64 {
    let magnitude = 10f64.powf(raw.log10().floor());
    let n = raw / magnitude;
    let m = if n <= 1.0 {
        1.0
    } else if n <= 2.0 {
        2.0
    } else if n <= 2.5 {
        2.5
    } else if n <= 5.0 {
        5.0
    } else {
        10.0
    };
    m * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_round() {
        assert_eq!(nice_step(0.9), 1.0);
        assert_eq!(nice_step(1.7), 2.0);
        assert_eq!(nice_step(2.3), 2.5);
        assert_eq!(nice_step(4.0), 5.0);
        assert_eq!(nice_step(7.0), 10.0);
        assert_eq!(nice_step(13.0), 20.0);
        assert!((nice_step(0.07) - 0.1).abs() < 1e-12);
    }
}
