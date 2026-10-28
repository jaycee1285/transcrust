//! Read the small GTK palette used by the desktop review adapter.
use eframe::egui::{self, Color32, Stroke};
use std::collections::HashMap;

pub fn apply(ctx: &egui::Context) {
    let Some(path) = dirs::config_dir().map(|p| p.join("gtk-4.0/gtk.css")) else {
        return;
    };
    let Ok(css) = std::fs::read_to_string(path) else {
        return;
    };
    // Comments may contain old definitions; only live declarations count.
    let comments = regex::Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let css = comments.replace_all(&css, "");
    let declarations = regex::Regex::new(r"@define-color\s+([\w-]+)\s+([^;]+);").unwrap();
    let colors: HashMap<_, _> = declarations
        .captures_iter(&css)
        .map(|c| (c[1].to_owned(), c[2].trim().to_owned()))
        .collect();
    let get = |names: &[&str]| names.iter().find_map(|name| resolve(name, &colors, 0));
    let Some(bg) = get(&["window_bg_color", "theme_bg_color"]) else {
        return;
    };
    // GTK alpha foregrounds are flattened against their surface, rather than
    // interpreted as premultiplied RGB or dimmed again by egui.
    let dark = brightness(bg) < 128.0;
    let fg = over(
        get(&["window_fg_color", "theme_fg_color"]).unwrap_or(if dark {
            Color32::WHITE
        } else {
            Color32::BLACK
        }),
        bg,
    );
    let view = get(&["view_bg_color", "theme_base_color"]).unwrap_or(bg);
    let accent =
        get(&["accent_bg_color", "accent_color", "theme_selected_bg_color"]).map(|c| over(c, bg));
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = bg;
    visuals.window_fill = bg;
    visuals.extreme_bg_color = view;
    visuals.faint_bg_color = view;
    visuals.override_text_color = Some(fg);
    visuals.weak_text_color = Some(fg);
    let border = fg.lerp_to_gamma(bg, 0.65);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = view;
        widget.weak_bg_fill = view;
        widget.bg_stroke = Stroke::new(1.0, border);
        widget.fg_stroke.color = fg;
    }
    visuals.text_cursor.stroke.color = fg;
    if let Some(accent) = accent {
        let accent_fg = over(
            get(&["accent_fg_color", "theme_selected_fg_color"]).unwrap_or(
                if brightness(accent) < 128.0 {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                },
            ),
            accent,
        );
        visuals.selection.bg_fill = accent;
        visuals.selection.stroke = Stroke::new(1.0, accent_fg);
        visuals.widgets.hovered.bg_stroke.color = accent;
        visuals.widgets.active.bg_stroke.color = accent;
        visuals.widgets.open.bg_stroke.color = accent;
        visuals.hyperlink_color = accent;
    }
    ctx.set_theme(if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx.set_visuals(visuals);
}

fn resolve(name: &str, colors: &HashMap<String, String>, depth: usize) -> Option<Color32> {
    if depth > 8 {
        return None;
    }
    let value = colors.get(name)?.trim();
    if let Some(alias) = value.strip_prefix('@') {
        return resolve(alias, colors, depth + 1);
    }
    if let Some(hex) = value.strip_prefix('#') {
        let hex = if hex.len() == 3 || hex.len() == 4 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.to_owned()
        };
        if !hex.is_ascii() || !matches!(hex.len(), 6 | 8) {
            return None;
        }
        let byte = |i| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        return Some(Color32::from_rgba_unmultiplied(
            byte(0)?,
            byte(2)?,
            byte(4)?,
            if hex.len() == 8 { byte(6)? } else { 255 },
        ));
    }
    if let Some(inner) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
    {
        let values = inner
            .strip_suffix(')')?
            .split(',')
            .map(|v| v.trim().parse::<f32>().ok())
            .collect::<Option<Vec<_>>>()?;
        if !matches!(values.len(), 3 | 4) || values.iter().any(|v| !v.is_finite()) {
            return None;
        }
        return Some(Color32::from_rgba_unmultiplied(
            values[0].clamp(0.0, 255.0) as u8,
            values[1].clamp(0.0, 255.0) as u8,
            values[2].clamp(0.0, 255.0) as u8,
            (values.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0) * 255.0).round() as u8,
        ));
    }
    match value {
        "white" => Some(Color32::WHITE),
        "black" => Some(Color32::BLACK),
        "transparent" => Some(Color32::TRANSPARENT),
        _ => None,
    }
}

fn brightness(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}

fn over(fg: Color32, bg: Color32) -> Color32 {
    let [r, g, b, a] = fg.to_srgba_unmultiplied();
    let alpha = f32::from(a) / 255.0;
    let mix = |f, b| (f32::from(f) * alpha + f32::from(b) * (1.0 - alpha)).round() as u8;
    Color32::from_rgb(mix(r, bg.r()), mix(g, bg.g()), mix(b, bg.b()))
}
