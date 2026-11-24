//! Platform look-and-feel.
//!
//! The design carries two rounds — Fluent for Windows 11, Aqua for macOS — and
//! each platform keeps its own: accent, navigation, switch, window controls, how
//! a shortcut is spelled, what the pages are called.
//!
//! What is *not* platform-specific is the surface treatment. Both rounds of the
//! mockup outline every row in its own bordered card; the app instead uses one
//! frameless group per section with faint dividers inside it. That is a
//! deliberate departure, applied identically on both platforms.

use crate::entities::settings::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};
use crate::shared::i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
}

/// How the selected navigation entry is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavStyle {
    /// macOS: the whole row fills with the accent colour.
    Filled,
    /// Windows 11: a faint fill plus a short accent bar on the leading edge.
    Indicator,
}

/// Window controls drawn by the app, since these windows have no system chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionStyle {
    /// macOS: three dots, top left.
    TrafficLights,
    /// Windows: minimise and close glyphs, top right.
    Buttons,
}

/// A named face inside a system font file.
///
/// CJK fonts are shipped as collections, and the first face is usually a
/// document face whose metrics do not match the Latin UI font. [`Self::name`]
/// is looked up in the file so we load the UI face; if that name is not
/// present the candidate is skipped, rather than silently taking italic or a
/// body face. An empty name means "use face 0", for single-face files whose
/// typographic name we do not rely on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontFace {
    pub path: &'static str,
    pub name: &'static str,
}

const fn face(path: &'static str, name: &'static str) -> FontFace {
    FontFace { path, name }
}

#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_LATIN_UI: &[FontFace] = &[
    face(
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "DejaVu Sans",
    ),
    face(
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "Liberation Sans",
    ),
    face("/usr/share/fonts/noto/NotoSans-Regular.ttf", "Noto Sans"),
    face("/usr/share/fonts/TTF/DejaVuSans.ttf", "DejaVu Sans"),
    face("/usr/share/fonts/gnu-free/FreeSans.otf", "FreeSans"),
];

const WINDOWS_LATIN_UI: &[FontFace] = &[
    face(r"C:\Windows\Fonts\segoeui.ttf", "Segoe UI"),
    face(r"C:\Windows\Fonts\SegUIVar.ttf", "Segoe UI Variable"),
    face(r"C:\Windows\Fonts\tahoma.ttf", "Tahoma"),
];

const MACOS_LATIN_UI: &[FontFace] = &[
    face("/System/Library/Fonts/SFNS.ttf", ""),
    face("/System/Library/Fonts/SFNSText.ttf", ""),
    face("/System/Library/Fonts/Helvetica.ttc", "Helvetica"),
];

#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_MONO: &[FontFace] = &[
    face(
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "DejaVu Sans Mono",
    ),
    face(
        "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        "Liberation Mono",
    ),
    face(
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        "DejaVu Sans Mono",
    ),
];

const WINDOWS_MONO: &[FontFace] = &[
    face(r"C:\Windows\Fonts\consola.ttf", "Consolas"),
    face(r"C:\Windows\Fonts\cour.ttf", "Courier New"),
];

const MACOS_MONO: &[FontFace] = &[
    face("/System/Library/Fonts/SFNSMono.ttf", ""),
    face("/System/Library/Fonts/Menlo.ttc", "Menlo"),
];

#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_CHINESE_UI: &[FontFace] = &[
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK SC",
    ),
    face(
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK SC",
    ),
    face(
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK SC",
    ),
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf",
        "Noto Sans CJK SC",
    ),
    face(
        "/usr/share/fonts/truetype/noto-cjk/NotoSansCJKsc-Regular.otf",
        "Noto Sans CJK SC",
    ),
    face(
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "WenQuanYi Micro Hei",
    ),
];

const WINDOWS_CHINESE_UI: &[FontFace] = &[
    face(r"C:\Windows\Fonts\msyh.ttc", "Microsoft YaHei UI"),
    face(r"C:\Windows\Fonts\msyh.ttc", "Microsoft YaHei"),
    face(r"C:\Windows\Fonts\msjh.ttc", "Microsoft JhengHei UI"),
    face(r"C:\Windows\Fonts\msjh.ttc", "Microsoft JhengHei"),
    face(r"C:\Windows\Fonts\Deng.ttf", "DengXian"),
];

const MACOS_CHINESE_UI: &[FontFace] = &[
    face("/System/Library/Fonts/PingFang.ttc", "PingFang SC"),
    face(
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "Hiragino Sans GB",
    ),
];

#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_JAPANESE_UI: &[FontFace] = &[
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK JP",
    ),
    face(
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK JP",
    ),
    face(
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK JP",
    ),
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJKjp-Regular.otf",
        "Noto Sans CJK JP",
    ),
];

const WINDOWS_JAPANESE_UI: &[FontFace] = &[
    face(r"C:\Windows\Fonts\YuGothM.ttc", "Yu Gothic UI"),
    face(r"C:\Windows\Fonts\meiryo.ttc", "Meiryo UI"),
    face(r"C:\Windows\Fonts\YuGothR.ttc", "Yu Gothic UI Semilight"),
    face(r"C:\Windows\Fonts\YuGothM.ttc", "Yu Gothic Medium"),
    face(r"C:\Windows\Fonts\YuGothR.ttc", "Yu Gothic Regular"),
    face(r"C:\Windows\Fonts\meiryo.ttc", "Meiryo"),
    face(r"C:\Windows\Fonts\msgothic.ttc", "MS UI Gothic"),
];

const MACOS_JAPANESE_UI: &[FontFace] = &[
    face(
        "/System/Library/Fonts/Hiragino Sans W3.ttc",
        "Hiragino Sans",
    ),
    face(
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "Hiragino Sans",
    ),
    face(
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "Hiragino Sans GB",
    ),
];

#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_KOREAN_UI: &[FontFace] = &[
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK KR",
    ),
    face(
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK KR",
    ),
    face(
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "Noto Sans CJK KR",
    ),
    face(
        "/usr/share/fonts/opentype/noto/NotoSansCJKkr-Regular.otf",
        "Noto Sans CJK KR",
    ),
];

const WINDOWS_KOREAN_UI: &[FontFace] = &[face(r"C:\Windows\Fonts\malgun.ttf", "Malgun Gothic")];

const MACOS_KOREAN_UI: &[FontFace] = &[face(
    "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    "Apple SD Gothic Neo",
)];

impl Platform {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Windows
        }
    }

    pub const fn metrics(self) -> Metrics {
        match self {
            Platform::Windows => Metrics {
                surface_radius: 8.0,
                group_radius: 8.0,
                control_radius: 5.0,
                menu_radius: 8.0,
                row_height: 48.0,
                row_inset: 16.0,
                window_padding: 22.0,
                nav_width: 186.0,
                page_title_size: 19.0,
                toggle_size: (38.0, 19.0),
                toggle_dim_knob_when_off: true,
                nav_style: NavStyle::Indicator,
                caption: CaptionStyle::Buttons,
                row_subtitles: true,
                nav_search: true,
                shadow_alpha: 128,
            },
            Platform::MacOs => Metrics {
                surface_radius: 10.0,
                group_radius: 10.0,
                control_radius: 6.0,
                menu_radius: 8.0,
                row_height: 40.0,
                row_inset: 14.0,
                window_padding: 20.0,
                nav_width: 172.0,
                page_title_size: 15.0,
                toggle_size: (38.0, 22.0),
                toggle_dim_knob_when_off: false,
                nav_style: NavStyle::Filled,
                caption: CaptionStyle::TrafficLights,
                row_subtitles: false,
                nav_search: false,
                shadow_alpha: 90,
            },
        }
    }

    /// How a hotkey reads to a user of this platform: `⌥⇧T` versus `Alt+Shift+T`.
    ///
    /// `spaced` widens the Windows form to `Alt + Shift + T` for the key badge in
    /// the settings window, which is how Windows itself renders it there. Menus
    /// keep the compact form.
    pub fn format_hotkey_with(self, modifiers: u32, key_name: &str, spaced: bool) -> String {
        match self {
            Platform::MacOs => {
                let mut s = String::new();
                if modifiers & MOD_CONTROL != 0 {
                    s.push('⌃');
                }
                if modifiers & MOD_ALT != 0 {
                    s.push('⌥');
                }
                if modifiers & MOD_SHIFT != 0 {
                    s.push('⇧');
                }
                if modifiers & MOD_WIN != 0 {
                    s.push('⌘');
                }
                s.push_str(key_name);
                s
            }
            Platform::Windows => {
                let mut parts = Vec::new();
                if modifiers & MOD_CONTROL != 0 {
                    parts.push("Ctrl");
                }
                if modifiers & MOD_ALT != 0 {
                    parts.push("Alt");
                }
                if modifiers & MOD_SHIFT != 0 {
                    parts.push("Shift");
                }
                if modifiers & MOD_WIN != 0 {
                    parts.push("Win");
                }
                parts.push(key_name);
                parts.join(if spaced { " + " } else { "+" })
            }
        }
    }

    pub fn format_hotkey(self, modifiers: u32, key_name: &str) -> String {
        self.format_hotkey_with(modifiers, key_name, false)
    }

    /// The space bar, named the way the rest of the OS names it.
    pub fn space_key(self) -> &'static str {
        t("Space")
    }

    /// Latin / Cyrillic UI face, best first. Missing files are skipped.
    ///
    /// Segoe UI is preferred over Segoe UI Variable: both report the same typo
    /// metrics, but the variable font is a poorer match for egui's loader, and
    /// the classic face is what Microsoft YaHei UI was designed to sit next to.
    pub const fn latin_ui_fonts(self) -> &'static [FontFace] {
        #[cfg(all(unix, not(target_os = "macos")))]
        return LINUX_LATIN_UI;

        #[allow(unreachable_code)]
        match self {
            Platform::Windows => WINDOWS_LATIN_UI,
            Platform::MacOs => MACOS_LATIN_UI,
        }
    }

    /// Monospace face for key badges and size labels.
    pub const fn mono_fonts(self) -> &'static [FontFace] {
        #[cfg(all(unix, not(target_os = "macos")))]
        return LINUX_MONO;

        #[allow(unreachable_code)]
        match self {
            Platform::Windows => WINDOWS_MONO,
            Platform::MacOs => MACOS_MONO,
        }
    }

    /// Chinese UI faces, best first.
    ///
    /// Microsoft YaHei UI (and PingFang SC, Noto Sans CJK SC) include Latin
    /// glyphs with the same body as the hanzi, so a mixed label such as
    /// "登录 Windows 时启动" stays on one baseline. The document faces
    /// (plain YaHei, JhengHei) are only reached when the UI face is missing.
    pub const fn chinese_ui_fonts(self) -> &'static [FontFace] {
        #[cfg(all(unix, not(target_os = "macos")))]
        return LINUX_CHINESE_UI;

        #[allow(unreachable_code)]
        match self {
            Platform::Windows => WINDOWS_CHINESE_UI,
            Platform::MacOs => MACOS_CHINESE_UI,
        }
    }

    /// Japanese UI faces, best first.
    ///
    /// Yu Gothic UI Regular shares typo metrics with Segoe UI, unlike Yu
    /// Gothic Regular, which reports a 1024-unit line gap and is what the
    /// stack used to pick as the first available CJK face on Chinese Windows
    /// (Meiryo is often not installed there).
    pub const fn japanese_ui_fonts(self) -> &'static [FontFace] {
        #[cfg(all(unix, not(target_os = "macos")))]
        return LINUX_JAPANESE_UI;

        #[allow(unreachable_code)]
        match self {
            Platform::Windows => WINDOWS_JAPANESE_UI,
            Platform::MacOs => MACOS_JAPANESE_UI,
        }
    }

    /// Korean UI faces, best first.
    pub const fn korean_ui_fonts(self) -> &'static [FontFace] {
        #[cfg(all(unix, not(target_os = "macos")))]
        return LINUX_KOREAN_UI;

        #[allow(unreachable_code)]
        match self {
            Platform::Windows => WINDOWS_KOREAN_UI,
            Platform::MacOs => MACOS_KOREAN_UI,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    /// Floating popups and windows.
    pub surface_radius: f32,
    /// The grouped settings container.
    pub group_radius: f32,
    /// Buttons, chips, text fields.
    pub control_radius: f32,
    pub menu_radius: f32,
    /// Height of a settings row.
    pub row_height: f32,
    /// Horizontal padding inside the grouped container.
    pub row_inset: f32,
    pub window_padding: f32,
    pub nav_width: f32,
    pub page_title_size: f32,
    pub toggle_size: (f32, f32),
    /// Fluent greys the knob out when the switch is off; Aqua keeps it white.
    pub toggle_dim_knob_when_off: bool,
    pub nav_style: NavStyle,
    pub caption: CaptionStyle,
    /// Windows 11 Settings pairs every row with a one-line explanation.
    pub row_subtitles: bool,
    /// Windows 11 Settings puts a search field above the navigation.
    pub nav_search: bool,
    pub shadow_alpha: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::settings::{MOD_ALT, MOD_SHIFT};

    #[test]
    fn mac_uses_glyphs_windows_uses_words() {
        assert_eq!(
            Platform::MacOs.format_hotkey(MOD_ALT | MOD_SHIFT, "T"),
            "⌥⇧T"
        );
        assert_eq!(
            Platform::Windows.format_hotkey(MOD_ALT | MOD_SHIFT, "T"),
            "Alt+Shift+T"
        );
    }

    #[test]
    fn the_windows_key_badge_is_spaced_out() {
        assert_eq!(
            Platform::Windows.format_hotkey_with(MOD_ALT | MOD_SHIFT, "T", true),
            "Alt + Shift + T"
        );
    }

    #[test]
    fn spacing_does_not_apply_to_the_mac_glyph_form() {
        assert_eq!(
            Platform::MacOs.format_hotkey_with(MOD_ALT | MOD_SHIFT, "T", true),
            "⌥⇧T"
        );
    }

    #[test]
    fn norepeat_and_other_stray_bits_do_not_appear() {
        // MOD_NOREPEAT is an OS-level flag, not something the user pressed.
        let with_norepeat = MOD_CONTROL | 0x4000;
        assert_eq!(
            Platform::Windows.format_hotkey(with_norepeat, "T"),
            "Ctrl+T"
        );
    }

    #[test]
    fn the_platform_identities_stay_distinct() {
        // The borderless treatment is shared, but everything that makes a
        // platform recognisable is not.
        let w = Platform::Windows.metrics();
        let m = Platform::MacOs.metrics();
        assert_ne!(w.nav_style, m.nav_style);
        assert_ne!(w.caption, m.caption);
        assert_ne!(w.row_subtitles, m.row_subtitles);
        assert_ne!(w.nav_search, m.nav_search);
        assert_ne!(w.toggle_dim_knob_when_off, m.toggle_dim_knob_when_off);
    }

    #[test]
    fn windows_chinese_ui_fonts_start_with_yahei_ui() {
        let fonts = Platform::Windows.chinese_ui_fonts();
        assert_eq!(
            fonts[0],
            face(r"C:\Windows\Fonts\msyh.ttc", "Microsoft YaHei UI")
        );
        assert!(fonts.iter().any(|f| f.name == "Microsoft JhengHei UI"));
        // Document faces come after their UI counterparts, never first.
        let ui = fonts
            .iter()
            .position(|f| f.name == "Microsoft YaHei UI")
            .unwrap();
        let body = fonts
            .iter()
            .position(|f| f.name == "Microsoft YaHei")
            .unwrap();
        assert!(ui < body);
    }

    #[test]
    fn windows_latin_prefers_classic_segoe_over_the_variable_font() {
        let fonts = Platform::Windows.latin_ui_fonts();
        assert_eq!(fonts[0].name, "Segoe UI");
        assert!(fonts.iter().any(|f| f.name == "Segoe UI Variable"));
    }

    #[test]
    fn windows_japanese_prefers_yu_gothic_ui_over_the_document_face() {
        let fonts = Platform::Windows.japanese_ui_fonts();
        assert_eq!(fonts[0].name, "Yu Gothic UI");
        let ui = fonts.iter().position(|f| f.name == "Yu Gothic UI").unwrap();
        let body = fonts
            .iter()
            .position(|f| f.name == "Yu Gothic Regular")
            .unwrap();
        assert!(ui < body);
    }

    #[test]
    fn windows_korean_uses_malgun_gothic() {
        assert_eq!(Platform::Windows.korean_ui_fonts()[0].name, "Malgun Gothic");
    }
}
