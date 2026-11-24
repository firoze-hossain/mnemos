//! crates/mnemos-core/src/lib.rs
//!
//! Mnemos Core: The Sovereign Clipboard Memory Engine
//!
//! Provides data models, deduplication, pin management, fuzzy search,
//! inline metadata detection, and reactive clip hub.

pub mod config;
pub mod history;
pub mod hub;
pub mod item;
pub mod search;

pub use config::{MnemosConfig, PopupPosition, ThemeMode};
pub use history::ClipHistory;
pub use hub::{MnemosHub, MnemosHub as ClipHub};
pub use item::{ClipItem, ItemKind};
pub use search::{rank_items, FuzzyMatcher};
