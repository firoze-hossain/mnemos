//! crates/mnemos-core/src/hub.rs
//!
//! Central coordinator for Mnemos: clipboard monitoring, search queries,
//! keyboard-driven navigation, item pinning, and paste dispatching.

use crate::config::MnemosConfig;
use crate::history::ClipHistory;
use crate::item::ClipItem;

#[derive(Clone, Debug)]
pub struct MnemosHub {
    pub history: ClipHistory,
    pub config: MnemosConfig,
    pub search_query: String,
    pub selected_idx: usize,
    pub is_open: bool,
    pub show_preview: bool,
    pub show_preferences: bool,
    pub preferences_tab: usize, // 0: General, 1: Appearance, 2: Pins, 3: Storage
    pub last_pasted_item: Option<ClipItem>,
    pub status_message: Option<String>,
}

impl Default for MnemosHub {
    fn default() -> Self {
        let mut hub = MnemosHub {
            history: ClipHistory::default(),
            config: MnemosConfig::default(),
            search_query: String::new(),
            selected_idx: 0,
            is_open: true,
            show_preview: true,
            show_preferences: false,
            preferences_tab: 0,
            last_pasted_item: None,
            status_message: None,
        };
        hub.populate_initial_maccy_demo();
        hub
    }
}

impl MnemosHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_maccy_demo() -> Self {
        Self::default()
    }

    /// Pre-populates realistic clipboard history directly synthesizing the macOS Maccy screenshot:
    /// Finder screenshots, cargo build commands, git repository URLs, inline color codes, etc.
    pub fn populate_initial_maccy_demo(&mut self) {
        let base_time = 1732435020; // 21 Nov 2026 10:37 AM

        self.history.push_text("550004560", "Terminal", base_time - 1800);
        self.history.push_text("#284058", "Figma", base_time - 1700);
        self.history.push_text("https://github.com/firoze-hossain/novaos.git", "Safari", base_time - 1600);
        self.history.push_text("550004750", "Terminal", base_time - 1500);
        self.history.push_text("select * from inv_stock_expi...product_code=550004750;", "DBNavigator", base_time - 1400);
        self.history.push_text("https://github.com/firoze-hossain/aero-browser.git", "Safari", base_time - 1300);
        self.history.push_text("aero-browser", "Terminal", base_time - 1200);
        self.history.push_text("uhldev@UNITEDs-MacBook...ownloaded ring v0.17.14\nauto-adjust enabled", "Terminal", base_time - 1100);
        self.history.push_text("Option 3: Add native macO...e know and we can set it up!", "Slack", base_time - 1000);
        self.history.push_text("cargo run", "Terminal", base_time - 900);
        self.history.push_text("https://github.com/firoze-hossain/zenith.git", "Safari", base_time - 800);
        self.history.push_text("https://github.com/firoze-hossain/vortex.git", "Safari", base_time - 700);
        self.history.push_text("https://github.com/firoze-hossain/vellum.git", "Safari", base_time - 600);
        self.history.push_text("https://github.com/firoze-hossain/spectra-player.git", "Safari", base_time - 500);
        self.history.push_text("cargo build", "Terminal", base_time - 400);
        self.history.push_text("cargo build --release", "Terminal", base_time - 300);

        // Top 9 Screenshot File Paths (as seen in Maccy top pins/history)
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 9.55.22 AM.png", "Finder", base_time - 250);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.09.22 AM.png", "Finder", base_time - 220);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 1", "Finder", base_time - 190);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.15.00 AM.png", "Finder", base_time - 160);
        self.history.push_text("file:///Users/uhldev/Desktop/Macos screenshot/", "Finder", base_time - 130);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.18.52 AM.png", "Finder", base_time - 100);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.22.56 AM.png", "Finder", base_time - 70);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.30.23 AM.png", "Finder", base_time - 40);
        self.history.push_text("file:///Users/uhldev/Desktop/Screenshot 2026-11-21 at 10.34.34 AM.png", "Finder", base_time);

        self.selected_idx = 0;
    }

    /// Returns ranked list of item indices matching current search query.
    pub fn current_filtered_indices(&self) -> Vec<usize> {
        self.history.search(&self.search_query)
    }

    /// Returns ranked list of item references matching current search query.
    pub fn filtered_items(&self) -> Vec<&ClipItem> {
        self.current_filtered_indices()
            .into_iter()
            .filter_map(|idx| self.history.items.get(idx))
            .collect()
    }

    /// Sets the search query and resets selection to 0.
    pub fn set_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.selected_idx = 0;
    }

    /// Returns reference to currently selected item.
    pub fn get_selected_item(&self) -> Option<&ClipItem> {
        let indices = self.current_filtered_indices();
        if self.selected_idx < indices.len() {
            let item_idx = indices[self.selected_idx];
            self.history.items.get(item_idx)
        } else {
            None
        }
    }

    pub fn move_up(&mut self) {
        let count = self.current_filtered_indices().len();
        if count > 0 {
            if self.selected_idx == 0 {
                self.selected_idx = count - 1;
            } else {
                self.selected_idx -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        let count = self.current_filtered_indices().len();
        if count > 0 {
            self.selected_idx = (self.selected_idx + 1) % count;
        }
    }

    /// Selects an item by shortcut index (1..9).
    pub fn select_shortcut(&mut self, number_1_to_9: usize) -> Option<ClipItem> {
        if number_1_to_9 >= 1 && number_1_to_9 <= 9 {
            let target_idx = number_1_to_9 - 1;
            let indices = self.current_filtered_indices();
            if target_idx < indices.len() {
                self.selected_idx = target_idx;
                return self.confirm_selection();
            }
        }
        None
    }

    /// Confirms current selection: copies and returns item for pasting.
    pub fn confirm_selection(&mut self) -> Option<ClipItem> {
        let indices = self.current_filtered_indices();
        if self.selected_idx < indices.len() {
            let item_idx = indices[self.selected_idx];
            let item = self.history.items[item_idx].clone();
            self.last_pasted_item = Some(item.clone());
            self.status_message = Some(format!("Pasted: {}", item.preview_line(24)));
            Some(item)
        } else {
            None
        }
    }

    /// Toggles pin on the currently selected item.
    pub fn toggle_pin_selected(&mut self) -> bool {
        let indices = self.current_filtered_indices();
        if self.selected_idx < indices.len() {
            let item_idx = indices[self.selected_idx];
            let success = self.history.toggle_pin(item_idx);
            // Re-bound selection index
            self.selected_idx = self.selected_idx.min(self.current_filtered_indices().len().saturating_sub(1));
            success
        } else {
            false
        }
    }

    /// Deletes the currently selected item.
    pub fn delete_selected(&mut self) -> bool {
        let indices = self.current_filtered_indices();
        if self.selected_idx < indices.len() {
            let item_idx = indices[self.selected_idx];
            let success = self.history.delete_item(item_idx);
            let new_count = self.current_filtered_indices().len();
            if self.selected_idx >= new_count && new_count > 0 {
                self.selected_idx = new_count - 1;
            }
            success
        } else {
            false
        }
    }

    /// Clears all unpinned history items.
    pub fn clear_unpinned(&mut self) {
        self.history.clear_unpinned();
        self.selected_idx = 0;
        self.status_message = Some("Cleared all unpinned clipboard history".to_string());
    }

    pub fn type_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.selected_idx = 0;
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.selected_idx = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.selected_idx = 0;
    }

    pub fn open_preferences(&mut self) {
        self.show_preferences = true;
    }

    pub fn close_preferences(&mut self) {
        self.show_preferences = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_lifecycle() {
        let mut hub = MnemosHub::default();
        assert!(!hub.history.items.is_empty());

        // First item is selected
        let first = hub.get_selected_item().unwrap();
        assert!(first.content.contains("Screenshot"));

        // Navigate down
        hub.move_down();
        assert_eq!(hub.selected_idx, 1);

        // Search query
        hub.type_search_char('c');
        hub.type_search_char('a');
        hub.type_search_char('r');
        hub.type_search_char('g');
        hub.type_search_char('o');

        let filtered = hub.current_filtered_indices();
        assert!(!filtered.is_empty());
        let matched = hub.get_selected_item().unwrap();
        assert!(matched.content.contains("cargo"));

        // Clear search
        hub.clear_search();
        assert_eq!(hub.search_query, "");
    }
}
