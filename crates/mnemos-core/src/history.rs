//! crates/mnemos-core/src/history.rs
//!
//! Clipboard History repository, deduplication, pin persistence, and privacy protection.

use crate::item::ClipItem;
use crate::search::rank_items;

#[derive(Clone, Debug)]
pub struct ClipHistory {
    pub items: Vec<ClipItem>,
    pub max_items: usize,
    pub next_id: u64,
    pub ignored_apps: Vec<String>,
}

impl Default for ClipHistory {
    fn default() -> Self {
        ClipHistory {
            items: Vec::new(),
            max_items: 200,
            next_id: 1,
            ignored_apps: vec![
                "1Password".to_string(),
                "Bitwarden".to_string(),
                "KeePassXC".to_string(),
                "Keychain Access".to_string(),
            ],
        }
    }
}

impl ClipHistory {
    pub fn new(max_items: usize) -> Self {
        let mut h = Self::default();
        h.max_items = max_items.max(10);
        h
    }

    /// Pushes a new text entry into clipboard history.
    /// Deduplicates identical content by bumping its copy count and timestamp.
    /// Respects ignored applications list and confidential flags.
    pub fn push_text(&mut self, text: &str, source_app: &str, timestamp: u64) -> Option<u64> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Privacy check: ignore credentials apps
        if self.is_ignored_app(source_app) {
            return None;
        }

        // Check for deduplication
        if let Some(pos) = self.items.iter().position(|i| i.content == text) {
            let mut existing = self.items.remove(pos);
            existing.copy_count += 1;
            existing.last_copied_at = timestamp;
            if !source_app.is_empty() {
                existing.source_app = source_app.to_string();
            }

            let id = existing.id;
            // Pinned items stay in pinned area, unpinned move to top of unpinned
            if existing.pinned {
                self.items.insert(pos, existing);
            } else {
                // Insert right after last pinned item (or at 0 if no pins)
                let insert_idx = self.first_unpinned_index();
                self.items.insert(insert_idx, existing);
            }
            return Some(id);
        }

        // Create new item
        let id = self.next_id;
        self.next_id += 1;
        let item = ClipItem::new(id, text, source_app, timestamp);

        let insert_idx = self.first_unpinned_index();
        self.items.insert(insert_idx, item);

        // Enforce max capacity by evicting oldest unpinned item
        self.enforce_capacity();

        Some(id)
    }

    /// Finds index where unpinned items start.
    pub fn first_unpinned_index(&self) -> usize {
        self.items.iter().position(|i| !i.pinned).unwrap_or(self.items.len())
    }

    /// Toggles pinned state of item at given index.
    pub fn toggle_pin(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            let was_pinned = self.items[index].pinned;
            self.items[index].pinned = !was_pinned;

            let item = self.items.remove(index);
            if item.pinned {
                // Move to top of pinned list
                self.items.insert(0, item);
            } else {
                // Move to top of unpinned list
                let insert_idx = self.first_unpinned_index();
                self.items.insert(insert_idx, item);
            }
            true
        } else {
            false
        }
    }

    /// Deletes item at given index.
    pub fn delete_item(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.items.remove(index);
            true
        } else {
            false
        }
    }

    /// Deletes all unpinned history items, preserving user pins.
    pub fn clear_unpinned(&mut self) {
        self.items.retain(|i| i.pinned);
    }

    /// Clears all history including pinned items.
    pub fn clear_all(&mut self) {
        self.items.clear();
    }

    pub fn is_ignored_app(&self, app: &str) -> bool {
        let app_lower = app.to_lowercase();
        self.ignored_apps.iter().any(|ign| app_lower.contains(&ign.to_lowercase()))
    }

    /// Returns ranked list of item indices matching `query`.
    pub fn search(&self, query: &str) -> Vec<usize> {
        rank_items(query, &self.items)
    }

    pub fn pinned_count(&self) -> usize {
        self.items.iter().filter(|i| i.pinned).count()
    }

    pub fn unpinned_count(&self) -> usize {
        self.items.iter().filter(|i| !i.pinned).count()
    }

    fn enforce_capacity(&mut self) {
        if self.items.len() > self.max_items {
            // Find and remove oldest unpinned item from the end
            if let Some(pos) = self.items.iter().rposition(|i| !i.pinned) {
                self.items.remove(pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_and_timestamps() {
        let mut history = ClipHistory::default();
        history.push_text("echo hello", "Terminal", 100);
        history.push_text("cargo build", "Terminal", 200);

        assert_eq!(history.items.len(), 2);
        assert_eq!(history.items[0].content, "cargo build");

        // Duplicate "echo hello"
        history.push_text("echo hello", "Terminal", 300);
        assert_eq!(history.items.len(), 2);
        assert_eq!(history.items[0].content, "echo hello");
        assert_eq!(history.items[0].copy_count, 2);
        assert_eq!(history.items[0].last_copied_at, 300);
    }

    #[test]
    fn test_pinning_and_clear_unpinned() {
        let mut history = ClipHistory::default();
        history.push_text("item 1", "App", 100);
        history.push_text("item 2", "App", 200);
        history.push_text("item 3", "App", 300);

        // Pin item 2 (which is currently at index 1)
        history.toggle_pin(1);
        assert!(history.items[0].pinned);
        assert_eq!(history.items[0].content, "item 2");

        // Clear unpinned
        history.clear_unpinned();
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].content, "item 2");
    }

    #[test]
    fn test_ignored_applications() {
        let mut history = ClipHistory::default();
        let res = history.push_text("super_secret_password", "1Password 8", 100);
        assert!(res.is_none());
        assert_eq!(history.items.len(), 0);
    }
}
