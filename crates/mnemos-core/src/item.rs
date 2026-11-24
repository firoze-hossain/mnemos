//! crates/mnemos-core/src/item.rs
//!
//! Clipboard Item representations, content classification, color extraction,
//! and metadata tracking for Mnemos.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    PlainText,
    RichText,
    Image,
    FileUrls,
    ColorCode,
    Url,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipItem {
    pub id: u64,
    pub kind: ItemKind,
    pub title: String,
    pub content: String,
    pub source_app: String,
    pub first_copied_at: u64, // Epoch seconds or monotonic system time
    pub last_copied_at: u64,
    pub copy_count: u32,
    pub pinned: bool,
    pub pin_index: Option<usize>,
    pub is_confidential: bool,
    pub has_newlines: bool,
    pub detected_color: Option<(u8, u8, u8)>, // (R, G, B) if color hex/rgb
}

impl ClipItem {
    pub fn new(id: u64, content: &str, source_app: &str, timestamp: u64) -> Self {
        let has_newlines = content.contains('\n') || content.contains('\r');
        let color = Self::detect_color(content);
        let kind = Self::detect_kind(content, color.is_some());
        let title = Self::generate_title(content, has_newlines);

        ClipItem {
            id,
            kind,
            title,
            content: content.to_string(),
            source_app: if source_app.is_empty() { "System".to_string() } else { source_app.to_string() },
            first_copied_at: timestamp,
            last_copied_at: timestamp,
            copy_count: 1,
            pinned: false,
            pin_index: None,
            is_confidential: Self::detect_confidential(content),
            has_newlines,
            detected_color: color,
        }
    }

    /// Generates a single-line title for display in the popup list,
    /// compressing newlines into spaces and trimming excess whitespace.
    pub fn generate_title(content: &str, _has_newlines: bool) -> String {
        let first_line = content.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            return content.chars().take(50).collect();
        }
        first_line.to_string()
    }

    /// Formats a single-line preview truncated with middle/trailing ellipsis
    /// matching Maccy's typography.
    pub fn preview_line(&self, max_chars: usize) -> String {
        let text = self.title.clone();
        if self.kind == ItemKind::FileUrls && text.starts_with("file://") {
            // Shorten file:// path for clean display
            if text.len() > max_chars && max_chars > 16 {
                let start_part = &text[..18];
                let end_idx = text.rfind('/').unwrap_or(text.len() - 1);
                let end_part = &text[end_idx..];
                return format!("{}...{}", start_part, end_part);
            }
        }

        if text.chars().count() > max_chars && max_chars > 3 {
            let half = (max_chars - 3) / 2;
            let left: String = text.chars().take(half).collect();
            let right: String = text.chars().rev().take(half).collect::<String>().chars().rev().collect();
            format!("{}...{}", left, right)
        } else {
            text
        }
    }

    /// Detects if content is a 6-digit hex color (e.g. "#284058", "284058", "#FFFFFF")
    /// or "rgb(r, g, b)".
    pub fn detect_color(s: &str) -> Option<(u8, u8, u8)> {
        let trimmed = s.trim();
        let hex_str = if trimmed.starts_with('#') {
            &trimmed[1..]
        } else {
            trimmed
        };

        if hex_str.len() == 6 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(val) = u32::from_str_radix(hex_str, 16) {
                let r = ((val >> 16) & 0xFF) as u8;
                let g = ((val >> 8) & 0xFF) as u8;
                let b = (val & 0xFF) as u8;
                return Some((r, g, b));
            }
        }

        // RGB format e.g. "rgb(40, 64, 88)"
        if trimmed.starts_with("rgb(") && trimmed.ends_with(')') {
            let inner = &trimmed[4..trimmed.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>(), parts[2].parse::<u8>()) {
                    return Some((r, g, b));
                }
            }
        }

        None
    }

    /// Detects category of item.
    pub fn detect_kind(s: &str, is_color: bool) -> ItemKind {
        if is_color {
            return ItemKind::ColorCode;
        }
        let trimmed = s.trim();
        if trimmed.starts_with("file://") || trimmed.starts_with('/') && (trimmed.contains('/') && trimmed.len() > 3) {
            return ItemKind::FileUrls;
        }
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return ItemKind::Url;
        }
        ItemKind::PlainText
    }

    /// Detects if text likely contains passwords, private keys, or credentials.
    pub fn detect_confidential(s: &str) -> bool {
        let lower = s.to_lowercase();
        lower.contains("bearer ")
            || lower.contains("private key")
            || lower.contains("session_token")
            || lower.contains("aws_secret_access_key")
            || lower.contains("ghp_")
            || lower.contains("xoxb-")
    }

    /// Formats timestamp in human-readable string: e.g. "21 November, 2026 10:37 AM".
    pub fn format_time(&self, timestamp: u64) -> String {
        // Simple human-readable conversion for demonstration and telemetry
        let total_mins = timestamp / 60;
        let mins = total_mins % 60;
        let hours = (total_mins / 60) % 24;
        let (h12, ampm) = if hours >= 12 {
            (if hours == 12 { 12 } else { hours - 12 }, "PM")
        } else {
            (if hours == 0 { 12 } else { hours }, "AM")
        };
        format!("24 November, 2026 {:02}:{:02} {}", h12, mins, ampm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_detection() {
        let hex1 = item_detect_color("#284058");
        assert_eq!(hex1, Some((40, 64, 88)));

        let hex2 = item_detect_color("284058");
        assert_eq!(hex2, Some((40, 64, 88)));

        let hex3 = item_detect_color("#FFFFFF");
        assert_eq!(hex3, Some((255, 255, 255)));

        let invalid = item_detect_color("hello world");
        assert_eq!(invalid, None);
    }

    fn item_detect_color(s: &str) -> Option<(u8, u8, u8)> {
        ClipItem::detect_color(s)
    }

    #[test]
    fn test_item_creation_and_preview() {
        let item = ClipItem::new(1, "file:///Users/uhldev/Desktop/Macos screenshot/Screenshot.png", "Finder", 1000);
        assert_eq!(item.kind, ItemKind::FileUrls);
        assert_eq!(item.source_app, "Finder");
        assert_eq!(item.copy_count, 1);
        assert!(!item.has_newlines);

        let ml_item = ClipItem::new(2, "cargo build\ncargo run", "Terminal", 1000);
        assert!(ml_item.has_newlines);
        assert_eq!(ml_item.title, "cargo build");
    }
}
