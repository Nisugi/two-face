use anyhow::Result;

/// Parse a hex color string like "#RRGGBB" into ratatui Color
pub fn parse_hex_color(hex: &str) -> Result<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(anyhow::anyhow!("Invalid hex color length"));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(ratatui::style::Color::Rgb(r, g, b))
}

pub fn color_to_hex_string(color: &crate::frontend::common::Color) -> Option<String> {
    // Color is now a simple RGB struct
    Some(format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b))
}

// OLD functions no longer needed after Phase 2 refactoring
#[allow(dead_code)]
pub(crate) fn _old_color_to_hex_string(color: &ratatui::style::Color) -> Option<String> {
    _old_color_to_rgb(color).map(|(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
}

#[allow(dead_code)]
pub(crate) fn _old_color_to_rgb(color: &ratatui::style::Color) -> Option<(u8, u8, u8)> {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => Some((*r, *g, *b)),
        Color::Indexed(index) => Some(indexed_color_to_rgb(*index)),
        Color::Reset => None,
        Color::Black => Some((0, 0, 0)),
        Color::Red => Some((205, 0, 0)),
        Color::Green => Some((0, 205, 0)),
        Color::Yellow => Some((205, 205, 0)),
        Color::Blue => Some((0, 0, 205)),
        Color::Magenta => Some((205, 0, 205)),
        Color::Cyan => Some((0, 205, 205)),
        Color::Gray => Some((192, 192, 192)),
        Color::DarkGray => Some((128, 128, 128)),
        Color::LightRed => Some((255, 102, 102)),
        Color::LightGreen => Some((144, 238, 144)),
        Color::LightYellow => Some((255, 255, 102)),
        Color::LightBlue => Some((173, 216, 230)),
        Color::LightMagenta => Some((255, 119, 255)),
        Color::LightCyan => Some((224, 255, 255)),
        Color::White => Some((255, 255, 255)),
    }
}

fn indexed_color_to_rgb(index: u8) -> (u8, u8, u8) {
    const STANDARD_COLORS: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    if index < 16 {
        return STANDARD_COLORS[index as usize];
    }

    if index <= 231 {
        let level = index as usize - 16;
        let r = level / 36;
        let g = (level % 36) / 6;
        let b = level % 6;
        let levels = [0, 95, 135, 175, 215, 255];
        return (levels[r], levels[g], levels[b]);
    }

    // Grayscale ramp
    let gray = 8 + (index.saturating_sub(232)) * 10;
    (gray, gray, gray)
}

pub fn blend_colors_hex(
    base: &crate::frontend::common::Color,
    target: &crate::frontend::common::Color,
    ratio: f32,
) -> Option<String> {
    // Color is now a simple RGB struct
    let (br, bg, bb) = (base.r, base.g, base.b);
    let (tr, tg, tb) = (target.r, target.g, target.b);
    let ratio = ratio.clamp(0.0, 1.0);
    let blend = |b: u8, t: u8| -> u8 {
        (b as f32 + (t as f32 - b as f32) * ratio)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Some(format!(
        "#{:02x}{:02x}{:02x}",
        blend(br, tr),
        blend(bg, tg),
        blend(bb, tb)
    ))
}

pub(crate) fn normalize_color(opt: &Option<String>) -> Option<String> {
    opt.as_ref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed == "-" {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[derive(Clone)]
pub struct WindowColors {
    pub border: Option<String>,
    pub background: Option<String>,
    pub text: Option<String>,
}

pub fn resolve_window_colors(
    base: &crate::config::WindowBase,
    theme: &crate::theme::AppTheme,
) -> WindowColors {
    let border =
        normalize_color(&base.border_color).or_else(|| color_to_hex_string(&theme.window_border));
    let background = if base.transparent_background {
        None
    } else {
        normalize_color(&base.background_color)
            .or_else(|| color_to_hex_string(&theme.window_background))
    };
    let text =
        normalize_color(&base.text_color).or_else(|| color_to_hex_string(&theme.text_primary));

    WindowColors {
        border,
        background,
        text,
    }
}
