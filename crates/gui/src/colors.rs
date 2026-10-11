use iced::{Color, Theme};

// Blend `to` over `from` by `factor` — the one primitive every derived shade
// in the app is built from.
pub fn mix(from: Color, to: Color, factor: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * factor,
        g: from.g + (to.g - from.g) * factor,
        b: from.b + (to.b - from.b) * factor,
        a: 1.0,
    }
}

pub fn dim(color: Color, factor: f32) -> Color {
    mix(Color::BLACK, color, factor)
}

pub fn accent(theme: &Theme) -> Color {
    theme.palette().primary
}

pub fn text(theme: &Theme) -> Color {
    theme.palette().text
}

pub fn secondary_text(theme: &Theme) -> Color {
    mix(theme.palette().background, theme.palette().text, 0.55)
}

pub fn chrome_background(theme: &Theme) -> Color {
    dim(theme.palette().background, 0.55)
}

pub fn group_background(theme: &Theme) -> Color {
    dim(theme.palette().background, 0.8)
}

pub fn divider_color(theme: &Theme) -> Color {
    mix(theme.palette().background, theme.palette().text, 0.1)
}

// Everything the fretboard canvas paints, derived from the theme palette so
// the board follows theme switches.
pub struct CanvasColors {
    pub background: Color,
    pub string: Color,
    pub nut: Color,
    pub fret_line: Color,
    pub inlay: Color,
    pub fret_number: Color,
    pub label: Color,
    pub accent: Color,
    pub field_label: Color,
    pub root_halo: Color,
}

impl CanvasColors {
    pub fn of(theme: &Theme) -> CanvasColors {
        let background = theme.palette().background;
        let text = theme.palette().text;
        CanvasColors {
            background,
            string: mix(background, text, 0.55),
            nut: mix(background, text, 0.8),
            fret_line: mix(background, text, 0.2),
            inlay: mix(background, text, 0.18),
            fret_number: mix(background, text, 0.5),
            label: text,
            accent: theme.palette().primary,
            field_label: mix(background, text, 0.65),
            root_halo: Color::BLACK,
        }
    }
}
