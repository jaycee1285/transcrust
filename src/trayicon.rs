//! Theme-independent tray icons.
//!
//! An SNI item can advertise its icon two ways: `IconName`, which the host
//! looks up in *its* icon theme, or `IconPixmap`, raw ARGB32 the host blits.
//! Name-only is the usual choice and it is why so many tray icons are wrong or
//! missing: the app has no say in, or knowledge of, the host's theme. When the
//! lookup fails the host silently substitutes a generic placeholder — the app
//! is never told. `emblem-ok-symbolic` is exactly that case here: a
//! freedesktop name that Adwaita does not ship, so two of this app's six states
//! rendered as a generic cog.
//!
//! So we publish both. `IconName` stays primary because hosts recolor themed
//! symbolic icons to match the panel, which a bitmap cannot do. These pixmaps
//! are the floor: whatever the host's theme is missing, it still gets a correct
//! glyph. They are drawn from signed distance fields at startup — no image
//! files to install, no decoder dependency, nothing to go stale.
//!
//! Bytes are `[A, R, G, B]` per pixel (ARGB32, network byte order) with
//! straight (un-premultiplied) alpha, which is what the spec asks for.

use std::sync::LazyLock;

use ksni::Icon;

use crate::state::AppState;

/// Sizes advertised to the host, which picks the closest at or above its
/// target. 22px is the common panel size; 44px covers HiDPI.
const SIZES: [i32; 2] = [22, 44];

/// Subsamples per axis when estimating edge coverage.
const SUPERSAMPLE: i32 = 4;

/// Half-width of the dark contour, in normalized units. The glyph is white
/// with a dark outline so it stays legible on both light and dark panels.
const OUTLINE: f32 = 0.13;

type Point = (f32, f32);

/// All six states rendered once at process start. `Tray::icon_pixmap` is called
/// on every property refresh, so this must not rasterize per call.
static ICONS: LazyLock<Vec<Vec<Icon>>> = LazyLock::new(|| {
    [
        AppState::Idle,
        AppState::Recording,
        AppState::Transcribing,
        AppState::Injecting,
        AppState::Complete,
        AppState::Error,
    ]
    .iter()
    .map(|state| render_all_sizes(*state))
    .collect()
});

pub fn for_state(state: AppState) -> Vec<Icon> {
    ICONS[index(state)].clone()
}

fn index(state: AppState) -> usize {
    match state {
        AppState::Idle => 0,
        AppState::Recording => 1,
        AppState::Transcribing => 2,
        AppState::Injecting => 3,
        AppState::Complete => 4,
        AppState::Error => 5,
    }
}

fn render_all_sizes(state: AppState) -> Vec<Icon> {
    SIZES.iter().map(|size| render(state, *size)).collect()
}

/// Signed distance to the glyph for `state`, in normalized `[-1, 1]` space.
/// Negative inside the shape.
fn distance(state: AppState, p: Point) -> f32 {
    match state {
        // Ready: a play triangle.
        AppState::Idle => sd_triangle(p, (-0.42, -0.60), (-0.42, 0.60), (0.58, 0.0)),
        // Capturing: a filled record dot.
        AppState::Recording => sd_circle(p, 0.55),
        // Working: an ellipsis.
        AppState::Transcribing => sd_circle((p.0 + 0.58, p.1), 0.17)
            .min(sd_circle(p, 0.17))
            .min(sd_circle((p.0 - 0.58, p.1), 0.17)),
        // Injecting and Complete: a checkmark. This is the pair that Adwaita
        // could not draw from `emblem-ok-symbolic`.
        AppState::Injecting | AppState::Complete => {
            sd_segment(p, (-0.60, 0.02), (-0.18, 0.44), 0.13)
                .min(sd_segment(p, (-0.18, 0.44), (0.60, -0.44), 0.13))
        }
        // Error: a cross.
        AppState::Error => sd_segment(p, (-0.45, -0.45), (0.45, 0.45), 0.13)
            .min(sd_segment(p, (-0.45, 0.45), (0.45, -0.45), 0.13)),
    }
}

fn render(state: AppState, size: i32) -> Icon {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    // One normalized unit spans half the icon, so this converts a distance in
    // normalized units to a distance in pixels for the antialiasing ramp.
    let to_pixels = size as f32 / 2.0;
    let step = 1.0 / SUPERSAMPLE as f32;

    for y in 0..size {
        for x in 0..size {
            let mut fill = 0.0f32;
            let mut outline = 0.0f32;

            for sub_y in 0..SUPERSAMPLE {
                for sub_x in 0..SUPERSAMPLE {
                    let px = x as f32 + (sub_x as f32 + 0.5) * step;
                    let py = y as f32 + (sub_y as f32 + 0.5) * step;
                    // Map pixel centre into [-1, 1].
                    let p = (px / to_pixels - 1.0, py / to_pixels - 1.0);
                    let d = distance(state, p);
                    if d <= 0.0 {
                        fill += 1.0;
                    }
                    if d - OUTLINE <= 0.0 {
                        outline += 1.0;
                    }
                }
            }

            let samples = (SUPERSAMPLE * SUPERSAMPLE) as f32;
            let fill = fill / samples;
            let outline = outline / samples;
            data.extend_from_slice(&composite(fill, outline));
        }
    }

    Icon {
        width: size,
        height: size,
        data,
    }
}

/// White fill over a dark contour, source-over with straight alpha.
fn composite(fill: f32, outline: f32) -> [u8; 4] {
    let alpha = fill + outline * (1.0 - fill);
    if alpha <= f32::EPSILON {
        return [0, 0, 0, 0];
    }
    // Fill is white (255) and the contour is black (0), so the numerator of the
    // source-over colour reduces to the fill term alone.
    let channel = (255.0 * fill / alpha).round().clamp(0.0, 255.0) as u8;
    [
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
        channel,
        channel,
        channel,
    ]
}

fn sd_circle(p: Point, radius: f32) -> f32 {
    (p.0 * p.0 + p.1 * p.1).sqrt() - radius
}

/// Distance to a thick line segment — the building block for the check and the
/// cross.
fn sd_segment(p: Point, a: Point, b: Point, thickness: f32) -> f32 {
    let (pax, pay) = (p.0 - a.0, p.1 - a.1);
    let (bax, bay) = (b.0 - a.0, b.1 - a.1);
    let len_sq = bax * bax + bay * bay;
    let t = if len_sq <= f32::EPSILON {
        0.0
    } else {
        ((pax * bax + pay * bay) / len_sq).clamp(0.0, 1.0)
    };
    let (dx, dy) = (pax - bax * t, pay - bay * t);
    (dx * dx + dy * dy).sqrt() - thickness
}

/// Exact distance for a convex triangle: the max of its three edge half-planes.
fn sd_triangle(p: Point, a: Point, b: Point, c: Point) -> f32 {
    let winding = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    let sign = if winding < 0.0 { -1.0 } else { 1.0 };
    half_plane(p, a, b, sign)
        .max(half_plane(p, b, c, sign))
        .max(half_plane(p, c, a, sign))
}

fn half_plane(p: Point, a: Point, b: Point, sign: f32) -> f32 {
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let length = (ex * ex + ey * ey).sqrt();
    if length <= f32::EPSILON {
        return 0.0;
    }
    // Outward normal for the given winding.
    sign * ((p.0 - a.0) * ey - (p.1 - a.1) * ex) / length
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATES: [AppState; 6] = [
        AppState::Idle,
        AppState::Recording,
        AppState::Transcribing,
        AppState::Injecting,
        AppState::Complete,
        AppState::Error,
    ];

    #[test]
    fn every_state_renders_every_advertised_size() {
        for state in STATES {
            let icons = for_state(state);
            assert_eq!(icons.len(), SIZES.len(), "{state:?} is missing a size");
            for icon in icons {
                assert!(SIZES.contains(&icon.width));
                assert_eq!(icon.width, icon.height);
                // ARGB32: four bytes per pixel, no padding, no stride.
                assert_eq!(icon.data.len(), (icon.width * icon.height * 4) as usize);
            }
        }
    }

    #[test]
    fn every_glyph_actually_marks_pixels() {
        // A silently blank pixmap is the failure this whole module exists to
        // prevent, so assert real coverage rather than just a correct length.
        for state in STATES {
            for icon in for_state(state) {
                let pixels = (icon.width * icon.height) as f32;
                let opaque = icon
                    .data
                    .chunks_exact(4)
                    .filter(|pixel| pixel[0] > 128)
                    .count() as f32;
                let coverage = opaque / pixels;
                assert!(
                    coverage > 0.02 && coverage < 0.95,
                    "{state:?} at {}px covers {coverage:.3} of the icon",
                    icon.width
                );
            }
        }
    }

    #[test]
    fn distinct_states_produce_distinct_glyphs() {
        // Idle and Recording sharing a bitmap would make the tray useless as a
        // status display while still passing every structural check above.
        for (left_index, left) in STATES.iter().enumerate() {
            for right in STATES.iter().skip(left_index + 1) {
                // Injecting and Complete are deliberately the same checkmark.
                if matches!(
                    (left, right),
                    (AppState::Injecting, AppState::Complete)
                        | (AppState::Complete, AppState::Injecting)
                ) {
                    continue;
                }
                assert_ne!(
                    for_state(*left)[0].data,
                    for_state(*right)[0].data,
                    "{left:?} and {right:?} render identically"
                );
            }
        }
    }

    #[test]
    fn alpha_is_straight_not_premultiplied() {
        // The host converts ARGB to RGBA and hands it to a Pixbuf, which
        // expects straight alpha. Premultiplied data would render as a dark
        // halo, so a fully opaque interior pixel must stay pure white.
        let icon = &for_state(AppState::Recording)[0];
        let centre = ((icon.height / 2) * icon.width + icon.width / 2) as usize * 4;
        let pixel = &icon.data[centre..centre + 4];
        assert_eq!(pixel, &[255, 255, 255, 255]);
    }
}
