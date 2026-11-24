//! Material Design 3–inspired design tokens.
//!
//! One coordinated light/dark system for every platform. The accent is a
//! restrained Google blue, chosen so labels on primary actions stay readable.
//! Surfaces are tonal: the window, the navigation pane and the grouped cards
//! sit a step apart, and a faint separator hints at structure without drawing
//! a hard outline on every row.
//!
//! Platform metrics (fonts, caption buttons, hotkey spelling) stay native.
//! Colour and typography no longer try to imitate Fluent or Aqua.

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Rounding, Stroke, Visuals};
use tracing::debug;

use crate::shared::i18n::{self, t, Lang};

use super::platform::{FontFace, Metrics, Platform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => t("Match the system"),
            ThemeMode::Light => t("Light"),
            ThemeMode::Dark => t("Dark"),
        }
    }

    pub fn resolve(self) -> bool {
        match self {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => system_prefers_dark(),
        }
    }
}

/// Reads the OS appearance setting. Falls back to light, which is what both
/// platforms ship with.
fn system_prefers_dark() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) =
            hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        {
            if let Ok(light) = key.get_value::<u32, _>("AppsUseLightTheme") {
                return light == 0;
            }
        }
        false
    }
    #[cfg(target_os = "macos")]
    {
        // `defaults read -g AppleInterfaceStyle` prints "Dark" and exits non-zero
        // when the system is in light mode.
        std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "Dark")
            .unwrap_or(false)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub platform: Platform,
    pub metrics: Metrics,

    // Surfaces, back to front.
    /// Settings window background.
    pub window: Color32,
    /// Navigation pane. Equal to `window` on Windows, where Settings uses one
    /// Mica surface throughout.
    pub chrome: Color32,
    /// The grouped container that settings rows sit in: a couple of tones off
    /// `window`, no outline.
    pub card: Color32,
    /// Translation column — a soft primary-container tint so the eye lands on it first.
    pub card_accent: Color32,
    /// Glass popups floating over the desktop.
    pub glass: Color32,
    /// Faint strip under a popup's action row.
    pub footer: Color32,
    /// Dimming laid over the frozen desktop during capture.
    pub scrim: Color32,

    /// Divider between rows inside a group. Deliberately at the edge of
    /// visibility — it should suggest a boundary, not draw one.
    pub separator: Color32,
    /// Reserved for surfaces that float over unknown content, where an edge is
    /// the only thing separating the panel from whatever is behind it.
    pub border: Color32,

    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    /// Foreground for anything painted on `accent`.
    pub on_accent: Color32,

    /// Brand mark and anything that says "this is the translation".
    /// Kept as a blue in the same family as `accent`, not a second brand colour.
    pub sakura: Color32,
    pub sakura_deep: Color32,
    pub sakura_soft: Color32,
    /// Primary actions and selection.
    pub accent: Color32,
    /// Tonal fill for selected navigation, chips and secondary buttons.
    pub primary_container: Color32,
    pub on_primary_container: Color32,
    /// Tonal container for destructive hover states (e.g. window close button).
    pub error_container: Color32,
    pub on_error_container: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

/// A colour at an opacity, premultiplied the way it is going to be blended.
///
/// The painter blends on the encoded values: what reaches the screen is
/// `colour × alpha + what was there × (1 − alpha)`, the same sum a browser or
/// a design tool does. "White at 8 %" is then what it says — a dark surface
/// lightened 8 % of the way to white.
///
/// `Color32::from_rgba_unmultiplied` multiplies in linear light and encodes
/// the result. That suits a renderer that blends in linear light and not this
/// one: white at 8 % arrived as bright as white at 30 %, and every "soft
/// wash" on a dark surface — a button, a card, a hairline — was a slab of mid
/// grey. Dark colours never showed it, which is how it went unnoticed.
pub fn wash(r: u8, g: u8, b: u8, alpha: u8) -> Color32 {
    fn scaled(value: u8, alpha: u8) -> u8 {
        ((value as u16 * alpha as u16 + 127) / 255) as u8
    }
    Color32::from_rgba_premultiplied(scaled(r, alpha), scaled(g, alpha), scaled(b, alpha), alpha)
}

impl Theme {
    pub fn resolve(mode: ThemeMode) -> Self {
        Self::for_platform(Platform::current(), mode.resolve())
    }

    pub fn for_platform(platform: Platform, dark: bool) -> Self {
        let mut theme = if dark {
            Self::material_dark()
        } else {
            Self::material_light()
        };
        theme.platform = platform;
        theme.metrics = platform.metrics();
        theme
    }

    fn material_light() -> Self {
        let platform = Platform::Windows;
        Self {
            dark: false,
            platform,
            metrics: platform.metrics(),

            window: Color32::from_rgb(0xF7, 0xF9, 0xFC),
            chrome: Color32::from_rgb(0xF0, 0xF4, 0xF9),
            card: Color32::WHITE,
            card_accent: Color32::from_rgb(0xD3, 0xE3, 0xFD),
            glass: wash(0xF7, 0xF9, 0xFC, 247),
            footer: wash(0, 0, 0, 8),
            scrim: wash(16, 22, 36, 148),

            separator: wash(0, 0, 0, 22),
            border: wash(0, 0, 0, 28),

            text: Color32::from_rgb(0x1A, 0x1C, 0x1E),
            text_dim: Color32::from_rgb(0x44, 0x47, 0x4F),
            text_faint: Color32::from_rgb(0x74, 0x77, 0x7F),
            on_accent: Color32::WHITE,

            sakura: Color32::from_rgb(0x0B, 0x57, 0xD0),
            sakura_deep: Color32::from_rgb(0x08, 0x42, 0xA0),
            sakura_soft: Color32::from_rgb(0x8A, 0xB4, 0xF8),
            accent: Color32::from_rgb(0x0B, 0x57, 0xD0),
            primary_container: Color32::from_rgb(0xD3, 0xE3, 0xFD),
            on_primary_container: Color32::from_rgb(0x04, 0x1E, 0x49),
            error_container: Color32::from_rgb(0xFF, 0xDA, 0xD6),
            on_error_container: Color32::from_rgb(0x93, 0x00, 0x0A),
            success: Color32::from_rgb(0x0F, 0x6B, 0x2B),
            warning: Color32::from_rgb(0x8C, 0x4A, 0x00),
            danger: Color32::from_rgb(0xB3, 0x26, 0x1E),
        }
    }

    fn material_dark() -> Self {
        let platform = Platform::Windows;
        Self {
            dark: true,
            platform,
            metrics: platform.metrics(),

            window: Color32::from_rgb(0x11, 0x13, 0x18),
            chrome: Color32::from_rgb(0x1A, 0x1C, 0x20),
            card: Color32::from_rgb(0x1E, 0x20, 0x25),
            card_accent: Color32::from_rgb(0x1B, 0x3F, 0x73),
            glass: wash(0x1E, 0x20, 0x25, 242),
            footer: wash(255, 255, 255, 10),
            scrim: wash(6, 8, 14, 150),

            separator: wash(255, 255, 255, 22),
            border: wash(255, 255, 255, 28),

            text: Color32::from_rgb(0xE3, 0xE2, 0xE6),
            text_dim: Color32::from_rgb(0xC4, 0xC7, 0xCF),
            text_faint: Color32::from_rgb(0x8E, 0x90, 0x99),
            on_accent: Color32::from_rgb(0x06, 0x2E, 0x6F),

            sakura: Color32::from_rgb(0xA8, 0xC7, 0xFA),
            sakura_deep: Color32::from_rgb(0xD3, 0xE3, 0xFD),
            sakura_soft: Color32::from_rgb(0x8A, 0xB4, 0xF8),
            accent: Color32::from_rgb(0xA8, 0xC7, 0xFA),
            primary_container: Color32::from_rgb(0x1B, 0x3F, 0x73),
            on_primary_container: Color32::from_rgb(0xD3, 0xE3, 0xFD),
            error_container: Color32::from_rgb(0x93, 0x00, 0x0A),
            on_error_container: Color32::from_rgb(0xFF, 0xDA, 0xD6),
            success: Color32::from_rgb(0x86, 0xDB, 0xA3),
            warning: Color32::from_rgb(0xF5, 0xBD, 0x6B),
            danger: Color32::from_rgb(0xFF, 0xB4, 0xAB),
        }
    }

    pub fn surface_rounding(&self) -> Rounding {
        Rounding::same(self.metrics.surface_radius)
    }

    /// The grouped settings container.
    pub fn group_rounding(&self) -> Rounding {
        Rounding::same(self.metrics.group_radius)
    }

    pub fn control_rounding(&self) -> Rounding {
        Rounding::same(self.metrics.control_radius)
    }

    pub fn border_stroke(&self) -> Stroke {
        Stroke::new(1.0_f32, self.border)
    }

    /// Same hue, dialled down — for hover fills and tinted notices.
    pub fn tint(&self, color: Color32, alpha: u8) -> Color32 {
        wash(color.r(), color.g(), color.b(), alpha)
    }

    /// Resting fill of a frameless control — a dropdown, a button, a key badge.
    /// Enough separation from the surface underneath to read as a target.
    pub fn control_fill(&self) -> Color32 {
        if self.dark {
            wash(255, 255, 255, 26)
        } else {
            wash(0, 0, 0, 16)
        }
    }

    pub fn control_fill_hover(&self) -> Color32 {
        if self.dark {
            wash(255, 255, 255, 42)
        } else {
            wash(0, 0, 0, 26)
        }
    }

    pub fn control_fill_active(&self) -> Color32 {
        if self.dark {
            wash(255, 255, 255, 58)
        } else {
            wash(0, 0, 0, 38)
        }
    }

    /// Fill for something you type into.
    ///
    /// A translucent wash is enough for a button, but not for a text field: with
    /// no outline to mark it, an input has to sit *below* its container to look
    /// editable at all. Recessed rather than raised, and opaque so it does not
    /// change with whatever it happens to be layered on.
    pub fn field_fill(&self) -> Color32 {
        if self.dark {
            Color32::from_rgb(0x0C, 0x0E, 0x13)
        } else {
            Color32::from_rgb(0xE7, 0xED, 0xF5)
        }
    }

    pub fn hover_fill(&self) -> Color32 {
        if self.dark {
            wash(255, 255, 255, 18)
        } else {
            wash(0, 0, 0, 12)
        }
    }

    /// Pushes the palette into egui so stock widgets match the custom ones.
    pub fn apply(&self, ctx: &Context) {
        let mut v = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.override_text_color = Some(self.text);
        v.panel_fill = self.window;
        v.window_fill = self.window;
        v.extreme_bg_color = self.field_fill();
        v.faint_bg_color = self.hover_fill();
        v.hyperlink_color = self.accent;
        v.window_rounding = self.surface_rounding();
        v.menu_rounding = Rounding::same(self.metrics.menu_radius);
        v.window_stroke = self.border_stroke();
        v.selection.bg_fill = self.tint(self.accent, 90);
        v.selection.stroke = Stroke::new(1.0_f32, self.accent);
        v.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 12.0),
            blur: 32.0,
            spread: 0.0,
            color: Color32::from_black_alpha(self.metrics.shadow_alpha),
        };
        v.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(self.metrics.shadow_alpha),
        };

        // Controls are tonal fills with a generous corner, not outlined boxes.
        // Dropdowns and text fields inherit the same rounding so the page
        // reads as one system.
        let r = self.control_rounding();
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.rounding = r;
            w.bg_stroke = Stroke::NONE;
            w.expansion = 0.0;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_dim);
        v.widgets.inactive.weak_bg_fill = self.control_fill();
        v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text);
        v.widgets.hovered.weak_bg_fill = self.control_fill_hover();
        v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, self.text);
        v.widgets.active.weak_bg_fill = self.control_fill_active();
        v.widgets.active.fg_stroke = Stroke::new(1.0_f32, self.text);
        v.widgets.open.weak_bg_fill = self.control_fill_active();
        v.widgets.open.fg_stroke = Stroke::new(1.0_f32, self.text);

        ctx.set_visuals(v);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 7.0);
        style.spacing.interact_size.y = 32.0;
        style.spacing.window_margin = egui::Margin::same(self.metrics.window_padding);
        style.visuals.clip_rect_margin = 0.0;
        ctx.set_style(style);
    }
}

// ── Fonts ────────────────────────────────────────────────────────────────────

/// Builds the font stack for the current interface language.
///
/// When the interface is Chinese, Japanese or Korean, the matching CJK *UI*
/// face goes first so hanzi/kana/hangul and the Latin mixed into the same
/// label (a product name, "Windows", a file size) share one body and one
/// baseline. Latin and Cyrillic interfaces keep the platform UI face first —
/// putting a CJK face in front is how "Русский" in the language list ended
/// up spaced like a ransom note, because those faces often carry full-width
/// Cyrillic.
///
/// CJK collections are asked for their UI face by name (Microsoft YaHei UI,
/// Yu Gothic UI, …), not for index 0. Index 0 is the document face: Yu
/// Gothic Regular reports a 1024-unit line gap, and the old stack applied a
/// downward `y_offset_factor` of 0.10–0.15 to compensate, which is what made
/// mixed labels look as if the characters were jumping up and down. UI faces
/// are used at their native metrics; no vertical tweak is applied.
pub fn build_fonts() -> FontDefinitions {
    let platform = Platform::current();
    let lang = i18n::current();
    let mut fonts = FontDefinitions::default();

    let mut loaded = Vec::new();
    for face in faces_for(platform, lang) {
        if let Some((key, data, label)) = load_face(face) {
            if fonts.font_data.contains_key(&key) {
                continue;
            }
            debug!(font = %label, path = %face.path, "font face loaded");
            fonts.font_data.insert(key.clone(), data);
            loaded.push(key);
        }
    }

    let mut mono_keys = Vec::new();
    for face in platform.mono_fonts().iter().copied() {
        if let Some((key, data, label)) = load_face(face) {
            if fonts.font_data.contains_key(&key) {
                continue;
            }
            debug!(font = %label, path = %face.path, "Monospace font loaded");
            fonts.font_data.insert(key.clone(), data);
            mono_keys.push(key);
        }
    }

    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
    for (i, key) in loaded.iter().enumerate() {
        prop.insert(i, key.clone());
    }

    let mono = fonts.families.entry(FontFamily::Monospace).or_default();
    let mut at = 0;
    for key in mono_keys.iter().chain(loaded.iter()) {
        if !mono.contains(key) {
            mono.insert(at, key.clone());
            at += 1;
        }
    }

    fonts
}

/// Preferred faces in family order for this platform and interface language.
///
/// After the primary face come the other scripts, so a glyph the primary
/// font does not have (Cyrillic in YaHei, hanzi in Segoe, hangul in either)
/// falls through instead of rendering tofu.
fn faces_for(platform: Platform, lang: Lang) -> Vec<FontFace> {
    let latin = platform.latin_ui_fonts();
    let zh = platform.chinese_ui_fonts();
    let ja = platform.japanese_ui_fonts();
    let ko = platform.korean_ui_fonts();
    let groups: &[&[FontFace]] = match lang {
        Lang::Zh => &[zh, latin, ja, ko],
        Lang::Ja => &[ja, latin, zh, ko],
        Lang::Ko => &[ko, latin, zh, ja],
        _ => &[latin, zh, ja, ko],
    };
    groups.iter().flat_map(|g| g.iter().copied()).collect()
}

/// A font file as egui wants it — borrowed, so that it is in memory once and
/// stays that way however often the stack is rebuilt. See
/// [`crate::shared::file_bytes`] for what handing egui an owned copy costs.
///
/// The key includes the face index, because the same `.ttc` can contribute
/// more than one face.
fn load_face(face: FontFace) -> Option<(String, FontData, String)> {
    let bytes = crate::shared::file_bytes::forever(face.path)?;
    let index = super::font_face::index_for_candidate(bytes, face.name)?;
    let mut data = FontData::from_static(bytes);
    data.index = index;
    let label = if face.name.is_empty() {
        format!("{}#{}", face.path, index)
    } else {
        format!("{}#{} ({})", face.path, index, face.name)
    };
    let key = format!("{}#{}", face.path, index);
    Some((key, data, label))
}

/// Type ramp. Kept in one place so a size change lands everywhere at once.
pub mod text {
    use egui::FontId;

    pub fn body() -> FontId {
        FontId::proportional(13.0)
    }
    /// Part of the type scale, kept so the ramp stays complete even when no
    /// screen currently reaches for this step.
    #[allow(dead_code)]
    pub fn body_strong() -> FontId {
        FontId::proportional(13.5)
    }
    pub fn small() -> FontId {
        FontId::proportional(12.0)
    }
    pub fn caption() -> FontId {
        FontId::proportional(11.0)
    }
    pub fn mono() -> FontId {
        FontId::monospace(12.0)
    }
    pub fn translation() -> FontId {
        FontId::proportional(14.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative luminance, for a WCAG-style contrast check.
    fn luminance(c: Color32) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (x, y) = (luminance(a), luminance(b));
        let (hi, lo) = if x > y { (x, y) } else { (y, x) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn all_themes() -> Vec<(&'static str, Theme)> {
        vec![
            (
                "windows light",
                Theme::for_platform(Platform::Windows, false),
            ),
            ("windows dark", Theme::for_platform(Platform::Windows, true)),
            ("mac light", Theme::for_platform(Platform::MacOs, false)),
            ("mac dark", Theme::for_platform(Platform::MacOs, true)),
        ]
    }

    #[test]
    fn a_label_on_a_primary_button_is_readable() {
        // Dark theme uses a light primary with a navy label; light theme uses
        // white on Google blue. Both must clear the 4.5:1 threshold.
        for (name, t) in all_themes() {
            let ratio = contrast(t.on_accent, t.accent);
            assert!(ratio >= 4.5, "{name}: accent contrast is only {ratio:.2}:1");
        }
    }

    #[test]
    fn body_text_is_readable_on_every_surface() {
        for (name, t) in all_themes() {
            for (surface_name, surface) in [("window", t.window), ("card", t.card)] {
                let ratio = contrast(t.text, surface);
                assert!(
                    ratio >= 7.0,
                    "{name}/{surface_name}: text contrast is only {ratio:.2}:1"
                );
            }
        }
    }

    #[test]
    fn dimmed_text_still_clears_the_minimum() {
        for (name, t) in all_themes() {
            let ratio = contrast(t.text_dim, t.card);
            assert!(
                ratio >= 4.0,
                "{name}: dimmed text contrast is only {ratio:.2}:1"
            );
        }
    }

    #[test]
    fn warning_and_danger_text_are_readable_on_a_card() {
        for (name, t) in all_themes() {
            for (label, color) in [("warning", t.warning), ("danger", t.danger)] {
                let ratio = contrast(color, t.card);
                assert!(
                    ratio >= 4.0,
                    "{name}/{label}: contrast is only {ratio:.2}:1"
                );
            }
        }
    }

    #[test]
    fn the_two_platforms_share_the_same_accent() {
        assert_eq!(
            Theme::for_platform(Platform::Windows, false).accent,
            Theme::for_platform(Platform::MacOs, false).accent
        );
        assert_eq!(
            Theme::for_platform(Platform::Windows, true).accent,
            Theme::for_platform(Platform::MacOs, true).accent
        );
    }

    #[test]
    fn the_translation_accent_is_the_same_on_both_platforms() {
        assert_eq!(
            Theme::for_platform(Platform::Windows, false).sakura,
            Theme::for_platform(Platform::MacOs, false).sakura
        );
    }

    #[test]
    fn a_label_on_a_tonal_container_is_readable() {
        for (name, t) in all_themes() {
            let ratio = contrast(t.on_primary_container, t.primary_container);
            assert!(
                ratio >= 4.5,
                "{name}: primary container contrast is only {ratio:.2}:1"
            );
            let err_ratio = contrast(t.on_error_container, t.error_container);
            assert!(
                err_ratio >= 4.5,
                "{name}: error container contrast is only {err_ratio:.2}:1"
            );
        }
    }
}
