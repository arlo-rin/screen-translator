//! Shared widget kit for the Material-inspired Screen Translator chrome.
//!
//! Surfaces are tonal fills with generous rounding. Selected chips, buttons,
//! and navigation use a restrained accent and container tint. Outlines appear
//! only where they mark state (an unselected chip, a floating overlay, a
//! recording hotkey). Platform identity stays in typography metrics, captions,
//! and how a shortcut is spelled — not in a second colour system.

use egui::{Align2, Color32, Painter, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2};

use super::platform::CaptionStyle;
use super::theme::{text, wash, Theme};
use crate::shared::i18n::t;

pub type IconFn = fn(&Painter, Rect, Color32);

// ── Containers ───────────────────────────────────────────────────────────────

/// A grouped container: one rounded fill holding a run of rows, no outline.
pub fn card<R>(ui: &mut Ui, theme: &Theme, add: impl FnOnce(&mut Ui) -> R) -> R {
    // Pinned to the available width. Without this the frame sizes to its
    // content, so a row measuring `available_width()` from inside it sees the
    // parent's width instead of the group's.
    let width = ui.available_width();
    egui::Frame::none()
        .fill(theme.card)
        .rounding(theme.group_rounding())
        .inner_margin(egui::Margin::symmetric(0.0, 4.0))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui)
        })
        .inner
}

/// A settings list. Identical on both platforms: one container, dividers inside.
pub fn list<R>(ui: &mut Ui, theme: &Theme, add: impl FnOnce(&mut Ui) -> R) -> R {
    card(ui, theme, add)
}

/// Frosted panel used by everything that floats over the desktop. The one place
/// a hairline survives — over an arbitrary screenshot there is nothing else to
/// separate the panel from the page underneath.
pub fn glass_frame(theme: &Theme) -> egui::Frame {
    egui::Frame::none()
        .fill(theme.glass)
        .rounding(theme.surface_rounding())
        .stroke(Stroke::new(1.0_f32, theme.border))
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 6.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(110),
        })
        .inner_margin(egui::Margin::same(0.0))
}

/// `ЯЗЫКИ` — the small all-caps label above a group.
pub fn section_caption(ui: &mut Ui, theme: &Theme, label: &str) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(label)
            .font(text::caption())
            .color(theme.text_dim)
            .strong(),
    );
    ui.add_space(8.0);
}

// ── Rows ─────────────────────────────────────────────────────────────────────

/// One entry in a settings list.
#[derive(Default)]
pub struct RowSpec<'a> {
    pub icon: Option<IconFn>,
    /// Colour for that icon. Defaults to the dimmed text colour, which is
    /// right for the interface's own glyphs and wrong for a brand mark: a
    /// grey Binance logo is not the Binance logo.
    pub icon_tint: Option<Color32>,
    pub title: &'a str,
    /// One line of explanation. Shown on Windows, where Settings always pairs a
    /// control with one; dropped on macOS, where it does not.
    pub subtitle: Option<&'a str>,
    /// Suppresses the divider under the final row of a group.
    pub last: bool,
    /// Tints the row — used while recording a shortcut.
    pub highlighted: bool,
}

impl<'a> RowSpec<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            ..Default::default()
        }
    }

    pub fn icon(mut self, icon: IconFn) -> Self {
        self.icon = Some(icon);
        self
    }

    #[allow(dead_code)]
    pub fn icon_tint(mut self, tint: Color32) -> Self {
        self.icon_tint = Some(tint);
        self
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn last(mut self) -> Self {
        self.last = true;
        self
    }

    pub fn highlighted(mut self, on: bool) -> Self {
        self.highlighted = on;
        self
    }
}

/// Renders a list row: optional icon, title, optional subtitle, and whatever the
/// caller draws on the trailing edge.
pub fn row<R>(
    ui: &mut Ui,
    theme: &Theme,
    spec: RowSpec<'_>,
    right: impl FnOnce(&mut Ui) -> R,
) -> R {
    let m = theme.metrics;
    let show_subtitle = m.row_subtitles && spec.subtitle.is_some();
    let height = if show_subtitle {
        m.row_height + 6.0
    } else {
        m.row_height
    };
    let inset = m.row_inset;
    let (bg, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

    if spec.highlighted {
        ui.painter()
            .rect_filled(bg, Rounding::ZERO, theme.tint(theme.accent, 20));
    }

    let mut text_left = bg.min.x + inset;
    if let Some(icon) = spec.icon {
        let size = 17.0;
        let icon_rect = Rect::from_min_size(
            egui::pos2(text_left, bg.center().y - size / 2.0),
            Vec2::splat(size),
        );
        icon(
            ui.painter(),
            icon_rect,
            spec.icon_tint.unwrap_or(theme.text_dim),
        );
        text_left = icon_rect.max.x + 12.0;
    }

    // The text is painted rather than laid out, so its width has to be measured
    // to know where the trailing controls may start. Without this a wide control
    // is free to sit on top of the subtitle.
    let painter = ui.painter();
    let mut text_right = text_left;
    if show_subtitle {
        let title = painter.layout_no_wrap(spec.title.to_owned(), text::body(), theme.text);
        let subtitle = painter.layout_no_wrap(
            spec.subtitle.unwrap_or_default().to_owned(),
            text::caption(),
            theme.text_dim,
        );
        text_right += title.size().x.max(subtitle.size().x);
        painter.galley(
            egui::pos2(text_left, bg.center().y - 9.0 - title.size().y / 2.0),
            title,
            theme.text,
        );
        painter.galley(
            egui::pos2(text_left, bg.center().y + 9.0 - subtitle.size().y / 2.0),
            subtitle,
            theme.text_dim,
        );
    } else {
        let title = painter.layout_no_wrap(spec.title.to_owned(), text::body(), theme.text);
        text_right += title.size().x;
        painter.galley(
            egui::pos2(text_left, bg.center().y - title.size().y / 2.0),
            title,
            theme.text,
        );
    }

    // Never let the label squeeze the controls out entirely, and never let the
    // controls start before the label ends.
    let trailing_left = (text_right + 12.0).min(bg.max.x - inset - 90.0);
    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                egui::pos2(trailing_left.max(bg.min.x + inset), bg.min.y + 4.0),
                egui::pos2(bg.max.x - inset, bg.max.y - 4.0),
            ))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    content.spacing_mut().item_spacing.x = 8.0;
    let out = right(&mut content);

    if !spec.last {
        // Inset on both sides so the divider reads as internal structure rather
        // than as an edge of the container.
        ui.painter().hline(
            (bg.min.x + inset)..=(bg.max.x - inset),
            bg.max.y,
            Stroke::new(1.0_f32, theme.separator),
        );
    }

    out
}

// ── Controls ─────────────────────────────────────────────────────────────────

/// The platform switch. Fluent greys the knob when off; Aqua keeps it white.
pub fn toggle(ui: &mut Ui, theme: &Theme, on: &mut bool) -> Response {
    let (w, h) = theme.metrics.toggle_size;
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }

    let how_on = ui.ctx().animate_bool_responsive(resp.id, *on);
    let p = ui.painter();
    let rounding = Rounding::same(h / 2.0);

    let track = if *on {
        if resp.hovered() {
            theme
                .accent
                .gamma_multiply(if theme.dark { 1.08 } else { 0.92 })
        } else {
            theme.accent
        }
    } else if resp.hovered() {
        theme.control_fill_hover()
    } else {
        Color32::TRANSPARENT
    };
    p.rect_filled(rect, rounding, track);
    if !*on {
        p.rect_stroke(
            rect,
            rounding,
            Stroke::new(
                2.0_f32,
                if resp.hovered() {
                    theme.text_dim
                } else {
                    theme.border
                },
            ),
        );
    }

    let pad = 3.0;
    let r = h / 2.0 - pad;
    let x = egui::lerp((rect.min.x + pad + r)..=(rect.max.x - pad - r), how_on);
    let knob = if *on {
        theme.on_accent
    } else if theme.dark {
        theme.text
    } else {
        Color32::WHITE
    };
    p.circle_filled(egui::pos2(x, rect.center().y), r, knob);
    if !*on {
        p.circle_stroke(
            egui::pos2(x, rect.center().y),
            r,
            Stroke::new(1.0_f32, theme.border),
        );
    }

    resp
}

/// Filled action button — "Копировать".
pub fn primary_button(ui: &mut Ui, theme: &Theme, label: &str) -> Response {
    button_impl(ui, theme, label, ButtonKind::Primary)
}

/// Quiet button on a soft fill.
pub fn secondary_button(ui: &mut Ui, theme: &Theme, label: &str) -> Response {
    button_impl(ui, theme, label, ButtonKind::Secondary)
}

/// Text-only button for tertiary actions.
pub fn ghost_button(ui: &mut Ui, theme: &Theme, label: &str) -> Response {
    button_impl(ui, theme, label, ButtonKind::Ghost)
}

#[derive(Clone, Copy, PartialEq)]
enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
}

fn button_impl(ui: &mut Ui, theme: &Theme, label: &str, kind: ButtonKind) -> Response {
    let enabled = ui.is_enabled();

    // A disabled action drops the accent entirely. Keeping the fill and dimming
    // the label produced a solid blue block with nothing legible in it — egui
    // already fades everything drawn through a disabled `Ui`, so dimming again
    // here pushed the text under the threshold.
    let (fill, fg) = match kind {
        _ if !enabled => (theme.control_fill(), theme.text_dim),
        ButtonKind::Primary => (theme.accent, theme.on_accent),
        ButtonKind::Secondary => (theme.primary_container, theme.on_primary_container),
        ButtonKind::Ghost => (Color32::TRANSPARENT, theme.accent),
    };

    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::small(), fg);
    let pad = Vec2::new(
        if kind == ButtonKind::Ghost {
            10.0
        } else {
            16.0
        },
        8.0,
    );
    let (rect, resp) = ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::click());

    let hot = resp.hovered() && enabled;
    let down = resp.is_pointer_button_down_on();
    let p = ui.painter();
    let bg = if !enabled {
        fill
    } else {
        match kind {
            ButtonKind::Primary => {
                if down {
                    fill.gamma_multiply(0.86)
                } else if hot {
                    fill.gamma_multiply(if theme.dark { 1.18 } else { 0.92 })
                } else {
                    fill
                }
            }
            ButtonKind::Secondary => {
                if down {
                    fill.gamma_multiply(if theme.dark { 1.16 } else { 0.92 })
                } else if hot {
                    fill.gamma_multiply(if theme.dark { 1.08 } else { 0.96 })
                } else {
                    fill
                }
            }
            ButtonKind::Ghost => {
                if down {
                    theme.tint(theme.accent, 36)
                } else if hot {
                    theme.tint(theme.accent, 20)
                } else {
                    Color32::TRANSPARENT
                }
            }
        }
    };

    if bg.a() > 0 {
        // Material buttons use a near-pill corner, larger than input fields.
        p.rect_filled(rect, Rounding::same(20.0), bg);
    }
    // `text` rather than `galley`: the colour is stated at the point of drawing,
    // so a disabled `Ui` cannot leave a filled box with an invisible label.
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        text::small(),
        fg,
    );

    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Square icon button: nothing at rest, a soft fill under the pointer.
pub fn icon_button(ui: &mut Ui, theme: &Theme, icon: IconFn, size: f32, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hot = resp.hovered();

    let p = ui.painter();
    let icon_rounding = Rounding::same((size * 0.5).min(20.0));
    if resp.is_pointer_button_down_on() {
        p.rect_filled(rect, icon_rounding, theme.tint(theme.accent, 36));
    } else if hot {
        p.rect_filled(rect, icon_rounding, theme.tint(theme.accent, 20));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let inset = rect.height() * 0.28;
    icon(
        p,
        Rect::from_center_size(rect.center(), Vec2::splat(rect.height() - inset * 2.0)),
        if hot { theme.text } else { theme.text_dim },
    );
    resp.on_hover_text(tip)
}

/// Language pill: `EN`.
pub fn chip(ui: &mut Ui, theme: &Theme, label: &str, accent: bool) -> Response {
    let fg = if accent {
        theme.on_primary_container
    } else {
        theme.text
    };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::caption(), fg);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + Vec2::new(16.0, 8.0), Sense::hover());

    let bg = if accent {
        theme.primary_container
    } else {
        theme.control_fill()
    };
    let p = ui.painter();
    let rounding = Rounding::same(rect.height() / 2.0);
    p.rect_filled(rect, rounding, bg);
    if !accent {
        p.rect_stroke(rect, rounding, Stroke::new(1.0_f32, theme.border));
    }
    p.galley(rect.center() - galley.size() / 2.0, galley, fg);
    resp
}

/// Monospace key badge: `Alt + Shift + T`.
pub fn hotkey_badge(ui: &mut Ui, theme: &Theme, label: &str, recording: bool) -> Response {
    let fg = if recording { theme.accent } else { theme.text };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::mono(), fg);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + Vec2::new(20.0, 9.0), Sense::click());

    let p = ui.painter();
    let rounding = theme.control_rounding();
    if recording {
        // The one dashed outline left in the app: "waiting for a keypress" is a
        // state no fill communicates.
        p.rect_filled(rect, rounding, theme.tint(theme.accent, 22));
        dashed_rect(p, rect, rounding, Stroke::new(1.2_f32, theme.accent));
    } else {
        let fill = if resp.hovered() {
            theme.control_fill_hover()
        } else {
            theme.control_fill()
        };
        p.rect_filled(rect, rounding, fill);
    }
    p.galley(rect.center() - galley.size() / 2.0, galley, fg);
    resp
}

/// Unbound shortcut: no box, just dimmed text.
pub fn unbound_badge(ui: &mut Ui, theme: &Theme, label: &str) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::mono(), theme.text_faint);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + Vec2::new(20.0, 9.0), Sense::click());
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        theme.text_faint,
    );
    resp
}

fn dashed_rect(p: &Painter, rect: Rect, rounding: Rounding, stroke: Stroke) {
    let dash = 4.0;
    let gap = 3.0;
    let inset = rounding.nw;
    let mut x = rect.min.x + inset;
    while x < rect.max.x - inset {
        let x2 = (x + dash).min(rect.max.x - inset);
        p.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x2, rect.min.y)],
            stroke,
        );
        p.line_segment(
            [Pos2::new(x, rect.max.y), Pos2::new(x2, rect.max.y)],
            stroke,
        );
        x += dash + gap;
    }
    let mut y = rect.min.y + inset;
    while y < rect.max.y - inset {
        let y2 = (y + dash).min(rect.max.y - inset);
        p.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.min.x, y2)],
            stroke,
        );
        p.line_segment(
            [Pos2::new(rect.max.x, y), Pos2::new(rect.max.x, y2)],
            stroke,
        );
        y += dash + gap;
    }
}

/// The capture-mode control at the top of the overlay. Returns the index that
/// was clicked.
///
/// The selected segment is a solid accent pill so it stays readable on the
/// dark overlay chrome, on any desktop background.
pub fn segmented(
    ui: &mut Ui,
    theme: &Theme,
    items: &[(IconFn, &str)],
    selected: usize,
) -> Option<usize> {
    let mut clicked = None;
    let selected_fill = theme.accent;
    let selected_fg = theme.on_accent;

    egui::Frame::none()
        .fill(OVERLAY_SURFACE)
        .rounding(theme.surface_rounding())
        .inner_margin(egui::Margin::same(4.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(|ui| {
                for (i, (icon, label)) in items.iter().enumerate() {
                    let on = i == selected;
                    let fg = if on {
                        selected_fg
                    } else {
                        wash(255, 255, 255, 210)
                    };
                    let galley =
                        ui.painter()
                            .layout_no_wrap((*label).to_owned(), text::small(), fg);
                    let icon_w = 14.0;
                    let size = Vec2::new(galley.size().x + icon_w + 28.0, 26.0);
                    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

                    let seg_rounding = Rounding::same(rect.height() / 2.0);
                    if on {
                        ui.painter().rect_filled(rect, seg_rounding, selected_fill);
                    } else if resp.hovered() {
                        ui.painter()
                            .rect_filled(rect, seg_rounding, wash(255, 255, 255, 28));
                    }

                    let icon_rect = Rect::from_min_size(
                        egui::pos2(rect.min.x + 9.0, rect.center().y - icon_w / 2.0),
                        Vec2::splat(icon_w),
                    );
                    icon(ui.painter(), icon_rect, fg);
                    ui.painter().galley(
                        egui::pos2(
                            icon_rect.max.x + 5.0,
                            rect.center().y - galley.size().y / 2.0,
                        ),
                        galley,
                        fg,
                    );

                    if resp.clicked() {
                        clicked = Some(i);
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
        });

    clicked
}

/// Dark chrome for controls that sit on the frozen desktop.
const OVERLAY_SURFACE: Color32 = Color32::from_rgba_premultiplied(38, 38, 42, 224);

/// A standalone action on the capture overlay, styled to match the mode strip.
pub fn overlay_button(ui: &mut Ui, theme: &Theme, icon: IconFn, label: &str) -> Response {
    let fg = wash(255, 255, 255, 228);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::small(), fg);
    let icon_w = 13.0;
    let size = Vec2::new(galley.size().x + icon_w + 32.0, 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let p = ui.painter();
    let fill = if resp.is_pointer_button_down_on() {
        wash(74, 74, 80, 240)
    } else if resp.hovered() {
        wash(60, 60, 66, 236)
    } else {
        OVERLAY_SURFACE
    };
    p.rect_filled(rect, theme.surface_rounding(), fill);

    let icon_rect = Rect::from_min_size(
        egui::pos2(rect.min.x + 12.0, rect.center().y - icon_w / 2.0),
        Vec2::splat(icon_w),
    );
    icon(p, icon_rect, fg);
    p.galley(
        egui::pos2(
            icon_rect.max.x + 6.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        fg,
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Navigation entry in the settings window: a full-pill Material Design 3 item
/// with a `primary_container` active indicator and tonal hover state layer.
pub fn sidebar_item(
    ui: &mut Ui,
    theme: &Theme,
    icon: IconFn,
    tile: Color32,
    label: &str,
    selected: bool,
) -> Response {
    let h = 36.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::click());

    let p = ui.painter();
    let rounding = Rounding::same(rect.height() / 2.0);

    if selected {
        p.rect_filled(rect, rounding, theme.primary_container);
    } else if resp.is_pointer_button_down_on() {
        p.rect_filled(rect, rounding, theme.tint(theme.accent, 28));
    } else if resp.hovered() {
        p.rect_filled(rect, rounding, theme.hover_fill());
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let icon_size = 18.0;
    let icon_rect = Rect::from_min_size(
        egui::pos2(rect.min.x + 14.0, rect.center().y - icon_size / 2.0),
        Vec2::splat(icon_size),
    );

    let fg = if selected {
        theme.on_primary_container
    } else {
        theme.text
    };
    let icon_color = if selected {
        theme.on_primary_container
    } else if tile == Color32::TRANSPARENT {
        theme.text_dim
    } else {
        tile
    };
    icon(p, icon_rect, icon_color);

    p.text(
        egui::pos2(icon_rect.max.x + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        text::body(),
        fg,
    );

    resp
}

/// Search field above the navigation. Returns true when the text changed.
pub fn search_field(ui: &mut Ui, theme: &Theme, query: &mut String) -> bool {
    let h = 36.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
    let fill = theme.field_fill();
    let rounding = Rounding::same(h / 2.0);
    ui.painter().rect_filled(rect, rounding, fill);
    if resp.hovered() {
        ui.painter()
            .rect_stroke(rect, rounding, Stroke::new(1.0_f32, theme.accent));
    }

    let icon_rect = Rect::from_min_size(
        egui::pos2(rect.min.x + 8.0, rect.center().y - 7.0),
        Vec2::splat(14.0),
    );
    super::icons::search(ui.painter(), icon_rect, theme.text_dim);

    let mut field = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                egui::pos2(icon_rect.max.x + 6.0, rect.min.y),
                egui::pos2(rect.max.x - 6.0, rect.max.y),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    field
        .add(
            egui::TextEdit::singleline(query)
                .frame(false)
                .desired_width(f32::INFINITY)
                .font(text::small())
                .hint_text(crate::shared::i18n::t("Search settings")),
        )
        .changed()
}

/// A selectable tile — translation engines, result presentations.
pub struct SelectCard<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub selected: bool,
    /// Draws a tick after the subtitle. Only meaningful for entries that need a
    /// credential — a ticked "без ключа" reads as nonsense.
    pub ready: bool,
    /// Zero means "fill the available width".
    pub width: f32,
    pub height: f32,
}

impl<'a> SelectCard<'a> {
    /// How tall a card is unless told otherwise.
    pub const HEIGHT: f32 = 52.0;

    pub fn new(title: &'a str, subtitle: &'a str) -> Self {
        Self {
            title,
            subtitle,
            selected: false,
            ready: false,
            width: 0.0,
            height: Self::HEIGHT,
        }
    }

    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }

    pub fn ready(mut self, on: bool) -> Self {
        self.ready = on;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

/// Selection reads as a tinted fill plus an accent bar on the leading edge — the
/// same language as the navigation, and no outline anywhere.
pub fn select_card(ui: &mut Ui, theme: &Theme, card: SelectCard<'_>) -> Response {
    let width = if card.width > 0.0 {
        card.width
    } else {
        ui.available_width()
    };
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(width.max(120.0), card.height), Sense::click());
    let p = ui.painter();
    let rounding = theme.group_rounding();

    let fill = if card.selected {
        theme.primary_container
    } else if resp.is_pointer_button_down_on() {
        theme.control_fill_active()
    } else if resp.hovered() {
        theme.control_fill_hover()
    } else {
        theme.control_fill()
    };
    p.rect_filled(rect, rounding, fill);

    if card.selected {
        p.rect_stroke(
            rect.shrink(0.5),
            rounding,
            Stroke::new(1.5_f32, theme.accent),
        );
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let left = rect.min.x + 15.0;
    p.text(
        egui::pos2(left, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        card.title,
        text::body(),
        if card.selected {
            theme.on_primary_container
        } else {
            theme.text
        },
    );
    let subtitle = p.text(
        egui::pos2(left, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        card.subtitle,
        text::caption(),
        if card.ready {
            theme.success
        } else {
            theme.text_dim
        },
    );
    if card.ready {
        super::icons::check(
            p,
            Rect::from_min_size(
                egui::pos2(subtitle.max.x + 4.0, subtitle.center().y - 6.0),
                Vec2::splat(12.0),
            ),
            theme.success,
        );
    }

    resp
}

/// Hairline used between free-standing sections.
pub fn separator(ui: &mut Ui, theme: &Theme) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0_f32, theme.separator),
    );
}

/// Rounded status pill anchored to the bottom of the capture overlay.
pub fn hint_pill(ui: &mut Ui, theme: &Theme, label: &str) {
    let fg = wash(255, 255, 255, 200);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), text::caption(), fg);
    let size = galley.size() + Vec2::new(26.0, 10.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let _ = theme;
    ui.painter()
        .rect_filled(rect, Rounding::same(14.0), wash(0, 0, 0, 115));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, fg);
}

// ── Material Design 3 window top bar & borderless resize ─────────────────────

/// Height of the Material Design 3 window top bar.
pub const TITLEBAR_HEIGHT: f32 = 44.0;

/// Window actions requested by [`settings_titlebar`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TitleBarAction {
    pub start_drag: bool,
    pub minimize: bool,
    pub toggle_maximize: bool,
    pub close: bool,
}

/// Edge and corner resize handles for a borderless resizable viewport.
///
/// Returns `true` if a resize started this frame so the title bar drag region
/// does not also fire `StartDrag`.
pub fn handle_window_resize(ctx: &egui::Context, maximized: bool) -> bool {
    if maximized || ctx.dragged_id().is_some() {
        return false;
    }
    let rect = ctx.screen_rect();
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return false;
    };
    let Some(dir) = resize_direction(pos, rect) else {
        return false;
    };

    let cursor = match dir {
        egui::ResizeDirection::North | egui::ResizeDirection::South => {
            egui::CursorIcon::ResizeVertical
        }
        egui::ResizeDirection::East | egui::ResizeDirection::West => {
            egui::CursorIcon::ResizeHorizontal
        }
        egui::ResizeDirection::NorthWest | egui::ResizeDirection::SouthEast => {
            egui::CursorIcon::ResizeNwSe
        }
        egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => {
            egui::CursorIcon::ResizeNeSw
        }
    };
    ctx.set_cursor_icon(cursor);

    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        return true;
    }
    false
}

fn resize_direction(pos: Pos2, rect: Rect) -> Option<egui::ResizeDirection> {
    const EDGE: f32 = 5.0;
    const CORNER: f32 = 12.0;

    let on_left = pos.x <= rect.min.x + EDGE;
    let on_right = pos.x >= rect.max.x - EDGE;
    let on_top = pos.y <= rect.min.y + EDGE;
    let on_bottom = pos.y >= rect.max.y - EDGE;

    if !(on_left || on_right || on_top || on_bottom) {
        return None;
    }

    let near_left = pos.x <= rect.min.x + CORNER;
    let near_right = pos.x >= rect.max.x - CORNER;
    let near_top = pos.y <= rect.min.y + CORNER;
    let near_bottom = pos.y >= rect.max.y - CORNER;

    use egui::ResizeDirection;
    match (near_left, near_right, near_top, near_bottom) {
        (true, _, true, _) => Some(ResizeDirection::NorthWest),
        (_, true, true, _) => Some(ResizeDirection::NorthEast),
        (true, _, _, true) => Some(ResizeDirection::SouthWest),
        (_, true, _, true) => Some(ResizeDirection::SouthEast),
        _ if on_top => Some(ResizeDirection::North),
        _ if on_bottom => Some(ResizeDirection::South),
        _ if on_left => Some(ResizeDirection::West),
        _ if on_right => Some(ResizeDirection::East),
        _ => None,
    }
}

/// Material Design 3 top bar for the settings window.
///
/// Shares the `theme.chrome` tonal surface with the navigation pane below and
/// replaces native sharp caption chrome with circular Material icon buttons
/// (or macOS traffic lights when `CaptionStyle::TrafficLights` is active).
pub fn settings_titlebar(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    subtitle: &str,
    maximized: bool,
    resizing: bool,
) -> TitleBarAction {
    let mut action = TitleBarAction::default();
    let (bar, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), TITLEBAR_HEIGHT),
        Sense::hover(),
    );

    let edge = if maximized { 0.0 } else { 5.0 };
    let drag_rect = if theme.metrics.caption == CaptionStyle::TrafficLights {
        let lights = super::icons::traffic_lights(
            ui.painter(),
            egui::pos2(bar.min.x + 16.0, bar.center().y - 5.5),
            5.5,
            7.0,
            true,
        );
        let step = 11.0 + 7.0;
        let close_r = Rect::from_min_size(lights.min, Vec2::splat(11.0));
        let min_r = Rect::from_min_size(lights.min + Vec2::new(step, 0.0), Vec2::splat(11.0));
        let max_r = Rect::from_min_size(lights.min + Vec2::new(step * 2.0, 0.0), Vec2::splat(11.0));

        if ui
            .interact(close_r, egui::Id::new("st_mac_close"), Sense::click())
            .on_hover_text(t("Close · Esc"))
            .clicked()
        {
            action.close = true;
        }
        if ui
            .interact(min_r, egui::Id::new("st_mac_min"), Sense::click())
            .on_hover_text(t("Minimize"))
            .clicked()
        {
            action.minimize = true;
        }
        if ui
            .interact(max_r, egui::Id::new("st_mac_max"), Sense::click())
            .on_hover_text(if maximized {
                t("Restore")
            } else {
                t("Maximize")
            })
            .clicked()
        {
            action.toggle_maximize = true;
        }

        paint_title_lockup(ui.painter(), bar, theme, title, subtitle, true);
        Rect::from_min_max(
            egui::pos2((lights.max.x + 10.0).min(bar.max.x), bar.min.y + edge),
            egui::pos2((bar.max.x - edge).max(bar.min.x), bar.max.y),
        )
    } else {
        let btn = 30.0;
        let gap = 6.0;
        let right_pad = 10.0;
        let btn_y = bar.center().y - btn / 2.0;

        let close_rect =
            Rect::from_min_size(egui::pos2(bar.max.x - right_pad - btn, btn_y), Vec2::splat(btn));
        let max_rect =
            Rect::from_min_size(egui::pos2(close_rect.min.x - gap - btn, btn_y), Vec2::splat(btn));
        let min_rect =
            Rect::from_min_size(egui::pos2(max_rect.min.x - gap - btn, btn_y), Vec2::splat(btn));

        if m3_caption_button(
            ui,
            theme,
            min_rect,
            "st_win_min",
            super::icons::minimize,
            t("Minimize"),
            false,
        )
        .clicked()
        {
            action.minimize = true;
        }
        if m3_caption_button(
            ui,
            theme,
            max_rect,
            "st_win_max",
            if maximized {
                super::icons::restore
            } else {
                super::icons::maximize
            },
            if maximized {
                t("Restore")
            } else {
                t("Maximize")
            },
            false,
        )
        .clicked()
        {
            action.toggle_maximize = true;
        }
        if m3_caption_button(
            ui,
            theme,
            close_rect,
            "st_win_close",
            super::icons::close,
            t("Close · Esc"),
            true,
        )
        .clicked()
        {
            action.close = true;
        }

        paint_title_lockup(ui.painter(), bar, theme, title, subtitle, false);
        Rect::from_min_max(
            egui::pos2(bar.min.x + edge, bar.min.y + edge),
            egui::pos2((min_rect.min.x - 6.0).max(bar.min.x + edge), bar.max.y),
        )
    };

    if drag_rect.width() > 0.0 && drag_rect.height() > 0.0 {
        let drag_resp = ui.interact(
            drag_rect,
            egui::Id::new("st_titlebar_drag"),
            Sense::click_and_drag(),
        );
        if !resizing {
            if drag_resp.double_clicked() {
                action.toggle_maximize = true;
            } else if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                action.start_drag = true;
            }
        }
    }

    action
}

fn paint_title_lockup(
    p: &Painter,
    bar: Rect,
    theme: &Theme,
    title: &str,
    subtitle: &str,
    centered: bool,
) {
    let icon_size = 18.0;
    let icon_gap = 10.0;

    let title_g = p.layout_no_wrap(title.to_owned(), text::body_strong(), theme.text);
    let sep_g = if subtitle.is_empty() {
        None
    } else {
        Some(p.layout_no_wrap("  ·  ".to_owned(), text::small(), theme.text_faint))
    };
    let sub_g = if subtitle.is_empty() {
        None
    } else {
        Some(p.layout_no_wrap(subtitle.to_owned(), text::small(), theme.text_dim))
    };

    let total_w = icon_size
        + icon_gap
        + title_g.size().x
        + sep_g.as_ref().map_or(0.0, |g| g.size().x)
        + sub_g.as_ref().map_or(0.0, |g| g.size().x);

    // On Windows/Linux, align the brand mark at x = 22.0 so it shares the exact
    // vertical axis of every icon in the navigation drawer (8 px margin + 14 px
    // item inset) and the title text lines up with every navigation label.
    let mut x = if centered {
        bar.center().x - total_w / 2.0
    } else {
        bar.min.x + 22.0
    };

    let icon_rect = Rect::from_min_size(
        egui::pos2(x, bar.center().y - icon_size / 2.0),
        Vec2::splat(icon_size),
    );
    super::icons::mark(p, icon_rect, theme.sakura);
    x = icon_rect.max.x + icon_gap;

    let title_y = bar.center().y - title_g.size().y / 2.0;
    let title_w = title_g.size().x;
    p.galley(egui::pos2(x, title_y), title_g, theme.text);
    x += title_w;

    if let (Some(sep), Some(sub)) = (sep_g, sub_g) {
        let sep_y = bar.center().y - sep.size().y / 2.0;
        let sep_w = sep.size().x;
        p.galley(egui::pos2(x, sep_y), sep, theme.text_faint);
        x += sep_w;

        let sub_y = bar.center().y - sub.size().y / 2.0;
        p.galley(egui::pos2(x, sub_y), sub, theme.text_dim);
    }
}

fn m3_caption_button(
    ui: &mut Ui,
    theme: &Theme,
    rect: Rect,
    id: &str,
    icon: IconFn,
    tip: &str,
    destructive: bool,
) -> Response {
    let resp = ui.interact(rect, egui::Id::new(id), Sense::click());
    let hot = resp.hovered();
    let down = resp.is_pointer_button_down_on();
    let p = ui.painter();
    let rounding = Rounding::same(rect.height() / 2.0);

    let (fill, fg) = if destructive {
        if down {
            (
                if theme.dark {
                    Color32::from_rgb(0xBA, 0x1A, 0x1A)
                } else {
                    theme.danger
                },
                Color32::WHITE,
            )
        } else if hot {
            (theme.error_container, theme.on_error_container)
        } else {
            (Color32::TRANSPARENT, theme.text_dim)
        }
    } else if down {
        (theme.control_fill_active(), theme.text)
    } else if hot {
        (theme.hover_fill(), theme.text)
    } else {
        (Color32::TRANSPARENT, theme.text_dim)
    };

    if fill.a() > 0 {
        p.rect_filled(rect, rounding, fill);
    }
    icon(p, rect.shrink(6.0), fg);
    if hot {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(tip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borderless_resize_detects_edges_corners_and_ignores_interior() {
        let r = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(820.0, 680.0));
        assert_eq!(
            resize_direction(Pos2::new(2.0, 2.0), r),
            Some(egui::ResizeDirection::NorthWest)
        );
        assert_eq!(
            resize_direction(Pos2::new(818.0, 3.0), r),
            Some(egui::ResizeDirection::NorthEast)
        );
        assert_eq!(
            resize_direction(Pos2::new(3.0, 678.0), r),
            Some(egui::ResizeDirection::SouthWest)
        );
        assert_eq!(
            resize_direction(Pos2::new(818.0, 678.0), r),
            Some(egui::ResizeDirection::SouthEast)
        );
        assert_eq!(
            resize_direction(Pos2::new(400.0, 2.0), r),
            Some(egui::ResizeDirection::North)
        );
        assert_eq!(
            resize_direction(Pos2::new(400.0, 678.0), r),
            Some(egui::ResizeDirection::South)
        );
        assert_eq!(
            resize_direction(Pos2::new(2.0, 300.0), r),
            Some(egui::ResizeDirection::West)
        );
        assert_eq!(
            resize_direction(Pos2::new(818.0, 300.0), r),
            Some(egui::ResizeDirection::East)
        );
        // Interior and caption button centers must never trigger a resize.
        assert_eq!(resize_direction(Pos2::new(400.0, 300.0), r), None);
        assert_eq!(resize_direction(Pos2::new(795.0, 22.0), r), None);
    }
}
