//! crates/mnemos-core/src/config.rs
//!
//! User preferences and system behavior settings for Mnemos.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupPosition {
    AtMouseCursor,
    AtMenuBar,
    ScreenCenter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MnemosConfig {
    pub max_history_size: usize,
    pub popup_position: PopupPosition,
    pub theme_mode: ThemeMode,
    pub paste_on_select: bool,
    pub play_sound: bool,
    pub show_preview_panel: bool,
    pub show_shortcuts: bool,
    pub fuzzy_search_enabled: bool,
    pub ignored_apps: Vec<String>,
}

impl Default for MnemosConfig {
    fn default() -> Self {
        MnemosConfig {
            max_history_size: 200,
            popup_position: PopupPosition::AtMouseCursor,
            theme_mode: ThemeMode::Dark,
            paste_on_select: true,
            play_sound: true,
            show_preview_panel: true,
            show_shortcuts: true,
            fuzzy_search_enabled: true,
            ignored_apps: vec![
                "1Password".to_string(),
                "Bitwarden".to_string(),
                "KeePassXC".to_string(),
                "Keychain Access".to_string(),
            ],
        }
    }
}
