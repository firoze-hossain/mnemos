//! crates/mnemos-ui/src/views.rs
//!
//! Visual layout and component rasterization for Mnemos:
//! - Main Translucent Floating Acrylic Card (Search, Pinned items, History, Shortcuts)
//! - Flyout Preview Popover with triangle pointer, source metadata, and full text
//! - Preferences Modal Dialog (General, Appearance, Storage)

use mnemos_core::hub::MnemosHub;
use mnemos_core::item::ItemKind;
use crate::draw::Surface;
use crate::font::{draw_char, draw_text, draw_text_small, Font};
use crate::geom::{Color, Rect};

pub const CARD_WIDTH: i32 = 360;
pub const CARD_ROW_HEIGHT: i32 = 28;
pub const MAX_VISIBLE_ROWS: usize = 12;
pub const PREVIEW_WIDTH: i32 = 340;
pub const PREVIEW_HEIGHT: i32 = 260;

pub struct MnemosRenderer;

impl MnemosRenderer {
    /// Renders the complete Mnemos interface onto the target surface.
    pub fn render<S: Surface + ?Sized>(s: &mut S, hub: &MnemosHub, screen_x: i32, screen_y: i32) {
        if !hub.is_open {
            return;
        }

        let filtered_items = hub.filtered_items();
        let visible_count = filtered_items.len().min(MAX_VISIBLE_ROWS);
        let header_height = 42;
        let footer_height = 100; // Clear, Preferences, About, Quit
        let list_height = (visible_count as i32) * CARD_ROW_HEIGHT;
        let card_height = header_height + list_height + footer_height + 12;

        let main_rect = Rect::new(screen_x, screen_y, CARD_WIDTH, card_height);

        // 1. Draw Main Acrylic Floating Window
        s.fill_rounded_rect(main_rect, 10, Color::MACCY_BG);
        s.draw_rounded_rect(main_rect, 10, Color::MACCY_BORDER);

        // 2. Search Bar Header
        let search_box = Rect::new(screen_x + 10, screen_y + 8, CARD_WIDTH - 20, 26);
        s.fill_rounded_rect(search_box, 6, Color::rgba(255, 255, 255, 12));
        s.draw_rounded_rect(search_box, 6, Color::MACCY_BORDER_SUBTLE);

        // Magnifying glass icon
        s.draw_search_icon(screen_x + 22, screen_y + 21, Color::TEXT_SECONDARY);

        // Search text / placeholder
        if hub.search_query.is_empty() {
            draw_text(s, screen_x + 36, screen_y + 14, "Type to search...", Color::TEXT_MUTED);
        } else {
            draw_text(s, screen_x + 36, screen_y + 14, &hub.search_query, Color::TEXT_PRIMARY);
            // Clear icon 'x'
            draw_text(s, screen_x + CARD_WIDTH - 26, screen_y + 14, "x", Color::TEXT_MUTED);
        }

        // Header separator
        let sep1_y = screen_y + header_height;
        s.draw_line(screen_x + 10, sep1_y, screen_x + CARD_WIDTH - 10, sep1_y, Color::MACCY_DIVIDER);

        // 3. Render Clipboard Item Rows
        let list_start_y = sep1_y + 4;
        let mut selected_row_rect: Option<Rect> = None;

        if filtered_items.is_empty() {
            draw_text(
                s,
                screen_x + 24,
                list_start_y + 10,
                "No matching clipboard history",
                Color::TEXT_MUTED,
            );
        } else {
            for (i, item) in filtered_items.iter().take(visible_count).enumerate() {
                let row_y = list_start_y + (i as i32) * CARD_ROW_HEIGHT;
                let row_rect = Rect::new(screen_x + 6, row_y, CARD_WIDTH - 12, CARD_ROW_HEIGHT - 2);
                let is_selected = i == hub.selected_idx;

                if is_selected {
                    selected_row_rect = Some(row_rect);
                    s.fill_rounded_rect(row_rect, 5, Color::SELECTION_BLUE);
                }

                let text_color = if is_selected { Color::TEXT_WHITE } else { Color::TEXT_PRIMARY };
                let shortcut_color = if is_selected { Color::rgb(220, 235, 255) } else { Color::TEXT_SHORTCUT };

                let mut cur_x = row_rect.x + 8;

                // Pin indicator
                if item.pinned {
                    draw_char(s, cur_x, row_rect.y + 7, '📌', Color::ACCENT_PIN, &Font::small());
                    cur_x += 14;
                }

                // Inline Color Swatch (if hex color detected)
                if let Some((r, g, b)) = item.detected_color {
                    let swatch_rect = Rect::new(cur_x, row_rect.y + 6, 14, 14);
                    s.fill_rounded_rect(swatch_rect, 3, Color::rgb(r, g, b));
                    s.draw_rounded_rect(swatch_rect, 3, Color::rgba(255, 255, 255, 80));
                    cur_x += 20;
                }

                // Item Title with middle truncation
                let available_chars = if hub.config.show_shortcuts && i < 9 { 26 } else { 32 };
                let preview_text = item.preview_line(available_chars);
                draw_text(s, cur_x, row_rect.y + 7, &preview_text, text_color);

                // Multiline symbol ↩
                if item.has_newlines {
                    draw_char(
                        s,
                        row_rect.x + row_rect.w - 58,
                        row_rect.y + 7,
                        '↩',
                        if is_selected { Color::TEXT_WHITE } else { Color::TEXT_MUTED },
                        &Font::regular(),
                    );
                }

                // Shortcut badge (1..9)
                if hub.config.show_shortcuts && i < 9 {
                    let shortcut_str = format!("{}⌘{}", if is_selected { "" } else { "⇧" }, i + 1);
                    draw_text_small(
                        s,
                        row_rect.x + row_rect.w - 42,
                        row_rect.y + 9,
                        &shortcut_str,
                        shortcut_color,
                    );
                }
            }
        }

        // 4. Footer Divider
        let footer_start_y = list_start_y + (visible_count.max(1) as i32) * CARD_ROW_HEIGHT + 4;
        s.draw_line(
            screen_x + 10,
            footer_start_y,
            screen_x + CARD_WIDTH - 10,
            footer_start_y,
            Color::MACCY_DIVIDER,
        );

        // 5. Footer Actions
        let f_actions = [
            ("Clear unpinned history", "⌥⇧⌘⌫", Color::ACCENT_DELETE),
            ("Preferences...", "⌘,", Color::TEXT_PRIMARY),
            ("About Mnemos", "", Color::TEXT_MUTED),
            ("Quit", "⌘Q", Color::TEXT_MUTED),
        ];

        for (fi, (label, shortcut, col)) in f_actions.iter().enumerate() {
            let fy = footer_start_y + 6 + (fi as i32) * 22;
            draw_text(s, screen_x + 16, fy, label, *col);
            if !shortcut.is_empty() {
                draw_text_small(
                    s,
                    screen_x + CARD_WIDTH - 64,
                    fy + 2,
                    shortcut,
                    Color::TEXT_SHORTCUT,
                );
            }
        }

        // 6. Flyout Preview Popover
        if hub.show_preview && hub.config.show_preview_panel && !filtered_items.is_empty() {
            if let Some(selected_item) = filtered_items.get(hub.selected_idx) {
                let preview_x = screen_x + CARD_WIDTH + 14;
                let preview_y = if let Some(sr) = selected_row_rect {
                    (sr.y - 40).max(screen_y).min(screen_y + card_height - PREVIEW_HEIGHT)
                } else {
                    screen_y + 10
                };

                let preview_rect = Rect::new(preview_x, preview_y, PREVIEW_WIDTH, PREVIEW_HEIGHT);

                // Callout triangle pointer pointing from popover left edge to selected row
                if let Some(sr) = selected_row_rect {
                    let tip_y = sr.center_y();
                    s.draw_popover_triangle(
                        preview_x - 10,
                        tip_y,
                        10,
                        0,
                        Color::PREVIEW_BG,
                        Color::MACCY_BORDER,
                    );
                }

                // Popover body
                s.fill_rounded_rect(preview_rect, 8, Color::PREVIEW_BG);
                s.draw_rounded_rect(preview_rect, 8, Color::MACCY_BORDER);

                // Header with application source
                let head_rect = Rect::new(preview_x, preview_y, PREVIEW_WIDTH, 34);
                s.fill_rounded_rect(head_rect, 8, Color::PREVIEW_HEADER);
                s.draw_line(
                    preview_x,
                    preview_y + 34,
                    preview_x + PREVIEW_WIDTH,
                    preview_y + 34,
                    Color::MACCY_DIVIDER,
                );

                let app_title = format!("App: {}", selected_item.source_app);
                draw_text(s, preview_x + 12, preview_y + 10, &app_title, Color::TEXT_PRIMARY);

                let kind_badge = match selected_item.kind {
                    ItemKind::PlainText => "Text",
                    ItemKind::FileUrls => "File",
                    ItemKind::ColorCode => "Color",
                    ItemKind::Url => "URL",
                    ItemKind::RichText => "Rich",
                    ItemKind::Image => "Image",
                };
                draw_text_small(
                    s,
                    preview_x + PREVIEW_WIDTH - 50,
                    preview_y + 12,
                    kind_badge,
                    Color::ACCENT_BLUE,
                );

                // Inset content container
                let inset_rect = Rect::new(preview_x + 10, preview_y + 44, PREVIEW_WIDTH - 20, 140);
                s.fill_rounded_rect(inset_rect, 6, Color::PREVIEW_INSET);
                s.draw_rounded_rect(inset_rect, 6, Color::MACCY_BORDER_SUBTLE);

                // Multiline content rendering
                let lines: Vec<&str> = selected_item.content.lines().collect();
                let max_lines = 8;
                for (li, line) in lines.iter().take(max_lines).enumerate() {
                    let ly = inset_rect.y + 8 + (li as i32) * 15;
                    let display_line: String = line.chars().take(38).collect();
                    draw_text_small(s, inset_rect.x + 8, ly, &display_line, Color::TEXT_PRIMARY);
                }
                if lines.len() > max_lines {
                    draw_text_small(
                        s,
                        inset_rect.x + 8,
                        inset_rect.y + 124,
                        format!("... ({} more lines)", lines.len() - max_lines).as_str(),
                        Color::TEXT_MUTED,
                    );
                }

                // Metadata bar
                let meta_y = preview_y + 192;
                let copies_str = format!("Copies: {}", selected_item.copy_count);
                let chars_str = format!("Chars: {}", selected_item.content.chars().count());
                draw_text_small(s, preview_x + 14, meta_y, &copies_str, Color::TEXT_SECONDARY);
                draw_text_small(s, preview_x + 100, meta_y, &chars_str, Color::TEXT_MUTED);

                if selected_item.pinned {
                    draw_text_small(s, preview_x + 180, meta_y, "Pinned", Color::ACCENT_PIN);
                }

                // Keyboard hints footer
                let hint_y = preview_y + 215;
                s.draw_line(
                    preview_x + 10,
                    hint_y - 4,
                    preview_x + PREVIEW_WIDTH - 10,
                    hint_y - 4,
                    Color::MACCY_DIVIDER,
                );
                draw_text_small(
                    s,
                    preview_x + 14,
                    hint_y + 2,
                    "⌥P (un)pin   ⌥⌫ delete   ↵ paste",
                    Color::TEXT_SHORTCUT,
                );
            }
        }

        // 7. Preferences Modal (if active)
        if hub.show_preferences {
            Self::render_preferences_dialog(s, screen_x + 20, screen_y + 30);
        }
    }

    /// Renders the Preferences modal dialog.
    fn render_preferences_dialog<S: Surface + ?Sized>(s: &mut S, px: i32, py: i32) {
        let pref_rect = Rect::new(px, py, 380, 280);
        s.fill_rounded_rect(pref_rect, 10, Color::rgb(32, 35, 44));
        s.draw_rounded_rect(pref_rect, 10, Color::rgba(255, 255, 255, 60));

        // Header
        draw_text(s, px + 18, py + 16, "Mnemos Preferences", Color::TEXT_WHITE);
        s.draw_line(px + 12, py + 40, px + 380 - 12, py + 40, Color::MACCY_DIVIDER);

        // Tabs
        let tabs = ["General", "Appearance", "Pins", "Storage"];
        for (ti, tab) in tabs.iter().enumerate() {
            let tx = px + 18 + (ti as i32) * 80;
            let is_sel = ti == 0;
            if is_sel {
                s.fill_rounded_rect(Rect::new(tx - 4, py + 46, 74, 22), 4, Color::rgba(255, 255, 255, 25));
            }
            draw_text_small(s, tx, py + 52, tab, if is_sel { Color::TEXT_WHITE } else { Color::TEXT_MUTED });
        }

        // Options
        let options = [
            ("Launch at login", "[✓] Enabled"),
            ("Paste automatically on select", "[✓] Enabled"),
            ("Play sound effects", "[✓] Enabled"),
            ("Show flyout preview popover", "[✓] Enabled"),
            ("Show keyboard shortcuts (1..9)", "[✓] Enabled"),
            ("Maximum history capacity", "200 items"),
            ("Ignored secure password managers", "1Password, Bitwarden"),
        ];

        for (oi, (key, val)) in options.iter().enumerate() {
            let oy = py + 80 + (oi as i32) * 22;
            draw_text_small(s, px + 20, oy, key, Color::TEXT_PRIMARY);
            draw_text_small(s, px + 230, oy, val, Color::ACCENT_BLUE);
        }

        // Close / Done button
        let done_btn = Rect::new(px + 290, py + 242, 70, 24);
        s.fill_rounded_rect(done_btn, 4, Color::SELECTION_BLUE);
        draw_text(s, px + 304, py + 246, "Done", Color::TEXT_WHITE);
    }
}
