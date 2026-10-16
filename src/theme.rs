//! Visual tokens: colors, typography, spacing. Light and dark.
//!
//! Direction: a financial terminal. Muted neutrals, exactly one accent,
//! dense data panels set in generous whitespace.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, Style,
    TextStyle, Theme, Visuals,
};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub accent: Color32,
    /// Text drawn on an accent fill.
    pub on_accent: Color32,
    pub danger: Color32,
}

pub const LIGHT: Palette = Palette {
    background: Color32::from_rgb(0xF3, 0xF1, 0xEC),
    surface: Color32::from_rgb(0xFB, 0xFA, 0xF7),
    border: Color32::from_rgb(0xD8, 0xD4, 0xCB),
    text_primary: Color32::from_rgb(0x1B, 0x1D, 0x20),
    text_secondary: Color32::from_rgb(0x6A, 0x68, 0x63),
    accent: Color32::from_rgb(0x23, 0x45, 0x6B),
    on_accent: Color32::from_rgb(0xFB, 0xFA, 0xF7),
    danger: Color32::from_rgb(0x9A, 0x2B, 0x2B),
};

pub const DARK: Palette = Palette {
    background: Color32::from_rgb(0x12, 0x14, 0x16),
    surface: Color32::from_rgb(0x1A, 0x1D, 0x20),
    border: Color32::from_rgb(0x2D, 0x31, 0x36),
    text_primary: Color32::from_rgb(0xE6, 0xE3, 0xDC),
    text_secondary: Color32::from_rgb(0x98, 0x95, 0x8D),
    accent: Color32::from_rgb(0x8F, 0xB0, 0xD4),
    on_accent: Color32::from_rgb(0x12, 0x14, 0x16),
    danger: Color32::from_rgb(0xDE, 0x7A, 0x72),
};

pub fn palette(dark_mode: bool) -> &'static Palette {
    if dark_mode { &DARK } else { &LIGHT }
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 40.0;
    pub const XXL: f32 = 64.0;
}

pub const SIDEBAR_WIDTH: f32 = 232.0;
pub const RADIUS: u8 = 3;

/// Font families beyond egui's built-in Proportional (Plex Sans) and Monospace (Plex Mono).
pub mod family {
    use egui::FontFamily;

    pub fn display() -> FontFamily {
        FontFamily::Name("display".into())
    }
    pub fn serif() -> FontFamily {
        FontFamily::Name("serif".into())
    }
    pub fn sans_medium() -> FontFamily {
        FontFamily::Name("sans-medium".into())
    }
    pub fn sans_semibold() -> FontFamily {
        FontFamily::Name("sans-semibold".into())
    }
    pub fn mono_medium() -> FontFamily {
        FontFamily::Name("mono-medium".into())
    }
}

/// Named text styles beyond egui's built-in ones.
pub mod text {
    use egui::TextStyle;

    /// Headline figures on the dashboard.
    pub fn display() -> TextStyle {
        TextStyle::Name("display".into())
    }
    /// Small uppercase labels above figures and sections.
    pub fn overline() -> TextStyle {
        TextStyle::Name("overline".into())
    }
    /// Navigation items.
    pub fn nav() -> TextStyle {
        TextStyle::Name("nav".into())
    }
}

macro_rules! font {
    ($file:literal) => {
        Arc::new(FontData::from_static(include_bytes!(concat!(
            "../assets/fonts/",
            $file
        ))))
    };
}

fn font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let embedded = [
        ("plex-serif-light", font!("IBMPlexSerif-Light.ttf")),
        ("plex-serif", font!("IBMPlexSerif-Regular.ttf")),
        ("plex-sans", font!("IBMPlexSans-Regular.ttf")),
        ("plex-sans-medium", font!("IBMPlexSans-Medium.ttf")),
        ("plex-sans-semibold", font!("IBMPlexSans-SemiBold.ttf")),
        ("plex-mono", font!("IBMPlexMono-Regular.ttf")),
        ("plex-mono-medium", font!("IBMPlexMono-Medium.ttf")),
    ];
    for (name, data) in embedded {
        fonts.font_data.insert(name.to_owned(), data);
    }

    // egui's bundled fonts stay on as fallbacks for any glyph Plex lacks.
    let fallbacks = fonts.families[&FontFamily::Proportional].clone();
    let with_fallbacks = |primary: &str| {
        let mut list = vec![primary.to_owned()];
        list.extend(fallbacks.iter().cloned());
        list
    };

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "plex-sans".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "plex-mono".to_owned());
    fonts
        .families
        .insert(family::display(), with_fallbacks("plex-serif-light"));
    fonts
        .families
        .insert(family::serif(), with_fallbacks("plex-serif"));
    fonts
        .families
        .insert(family::sans_medium(), with_fallbacks("plex-sans-medium"));
    fonts.families.insert(
        family::sans_semibold(),
        with_fallbacks("plex-sans-semibold"),
    );
    fonts
        .families
        .insert(family::mono_medium(), with_fallbacks("plex-mono-medium"));
    fonts
}

fn style_for(p: &Palette, dark: bool) -> Style {
    let mut style = Style {
        text_styles: [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, family::sans_medium())),
            (TextStyle::Heading, FontId::new(28.0, family::serif())),
            (
                TextStyle::Monospace,
                FontId::new(13.5, FontFamily::Monospace),
            ),
            (text::display(), FontId::new(88.0, family::display())),
            (text::overline(), FontId::new(11.0, family::sans_semibold())),
            (text::nav(), FontId::new(14.0, FontFamily::Proportional)),
        ]
        .into(),
        visuals: visuals_for(p, dark),
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(space::SM, space::SM);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.interact_size.y = 30.0;
    style
}

fn visuals_for(p: &Palette, dark: bool) -> Visuals {
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let radius = CornerRadius::same(RADIUS);
    v.panel_fill = p.background;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = radius;
    v.menu_corner_radius = radius;
    v.extreme_bg_color = p.surface;
    v.faint_bg_color = p.surface;
    v.code_bg_color = p.surface;
    v.override_text_color = Some(p.text_primary);
    v.weak_text_color = Some(p.text_secondary);
    v.hyperlink_color = p.accent;
    v.error_fg_color = p.danger;
    v.warn_fg_color = p.danger;
    v.selection.bg_fill = p.accent.gamma_multiply(0.25);
    v.selection.stroke = Stroke::new(1.0, p.accent);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text_primary);
    for (state, fill) in [
        (&mut w.inactive, p.surface),
        (&mut w.hovered, p.border.gamma_multiply(0.45)),
        (&mut w.active, p.border.gamma_multiply(0.7)),
        (&mut w.open, p.surface),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, p.border);
        state.fg_stroke = Stroke::new(1.0, p.text_primary);
        state.corner_radius = radius;
        state.expansion = 0.0;
    }
    w.hovered.bg_stroke = Stroke::new(1.0, p.text_secondary);
    v
}

/// Installs fonts and both light and dark styles. Call once at startup.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
    ctx.set_style_of(Theme::Light, style_for(&LIGHT, false));
    ctx.set_style_of(Theme::Dark, style_for(&DARK, true));
}

/// A bordered surface panel, used for figures and grouped content.
pub fn card(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(space::LG as i8))
}
