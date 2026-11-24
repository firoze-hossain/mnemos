//! crates/mnemos-core/src/search.rs
//!
//! Substring & Fuzzy ranking engine for instant clipboard search.

use crate::item::ClipItem;

pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Computes a fuzzy match score for `query` against `target`.
    /// Returns Some(score) if matched, where higher is better, or None if no match.
    pub fn score(query: &str, target: &str) -> Option<i32> {
        let q = query.trim().to_lowercase();
        let t = target.to_lowercase();

        if q.is_empty() {
            return Some(100);
        }

        if q.len() > t.len() {
            return None;
        }

        // 1. Exact match bonus
        if t == q {
            return Some(10_000);
        }

        // 2. Exact prefix match bonus
        if t.starts_with(&q) {
            return Some(5_000 + (100 - t.len().min(100) as i32));
        }

        // 3. Substring match bonus
        if let Some(pos) = t.find(&q) {
            let score = 2_000 - (pos as i32 * 10) + (q.len() as i32 * 50);
            return Some(score.max(1_000));
        }

        // 4. Sequential fuzzy character match with word boundary bonuses
        let q_chars: Vec<char> = q.chars().collect();
        let t_chars: Vec<char> = t.chars().collect();

        let mut q_idx = 0;
        let mut score = 0;
        let mut prev_matched_idx = -1i32;

        for (t_idx, &tc) in t_chars.iter().enumerate() {
            if q_idx < q_chars.len() && tc == q_chars[q_idx] {
                let mut char_score = 10;

                // Word boundary bonus: preceded by separator or start of word
                if t_idx == 0 {
                    char_score += 40;
                } else {
                    let prev_c = t_chars[t_idx - 1];
                    if prev_c == ' ' || prev_c == '/' || prev_c == '.' || prev_c == '_' || prev_c == '-' || prev_c == ':' {
                        char_score += 35;
                    }
                }

                // Consecutive match bonus
                if prev_matched_idx >= 0 && (t_idx as i32 == prev_matched_idx + 1) {
                    char_score += 25;
                }

                score += char_score;
                prev_matched_idx = t_idx as i32;
                q_idx += 1;
            }
        }

        if q_idx == q_chars.len() {
            // Deduct penalty for target length (prefer concise matches)
            let len_penalty = (t.len() as i32 - q.len() as i32).max(0);
            Some((score - len_penalty).max(10))
        } else {
            None
        }
    }
}

/// Ranks indices of `items` according to match score against `query`.
/// Pinned items retain priority boost.
pub fn rank_items(query: &str, items: &[ClipItem]) -> Vec<usize> {
    if query.trim().is_empty() {
        // Return in current order: pinned items first, then recent items
        let mut indices: Vec<usize> = (0..items.len()).collect();
        indices.sort_by(|&a, &b| {
            match (items[a].pinned, items[b].pinned) {
                (true, false) => core::cmp::Ordering::Less,
                (false, true) => core::cmp::Ordering::Greater,
                _ => items[b].last_copied_at.cmp(&items[a].last_copied_at),
            }
        });
        return indices;
    }

    let mut scored: Vec<(usize, i32)> = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        // Search in title, content, and source_app
        let score_title = FuzzyMatcher::score(query, &item.title);
        let score_content = FuzzyMatcher::score(query, &item.content);
        let score_app = FuzzyMatcher::score(query, &item.source_app);

        let best_score = [score_title, score_content, score_app]
            .into_iter()
            .flatten()
            .max();

        if let Some(mut score) = best_score {
            if item.pinned {
                score += 500; // Pinned items rank higher
            }
            scored.push((idx, score));
        }
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(idx, _)| idx).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matching() {
        let score_exact = FuzzyMatcher::score("cargo", "cargo build");
        assert!(score_exact.is_some());

        let score_prefix = FuzzyMatcher::score("cb", "cargo build");
        assert!(score_prefix.is_some());

        let score_none = FuzzyMatcher::score("zebra", "cargo build");
        assert!(score_none.is_none());
    }

    #[test]
    fn test_ranking() {
        let items = vec![
            ClipItem::new(1, "https://github.com/firoze-hossain/zenith.git", "Terminal", 100),
            ClipItem::new(2, "cargo build --release", "Terminal", 200),
            ClipItem::new(3, "file:///Users/uhldev/Desktop/Screenshot.png", "Finder", 300),
        ];

        let ranked_cargo = rank_items("cargo", &items);
        assert_eq!(ranked_cargo.len(), 1);
        assert_eq!(ranked_cargo[0], 1); // index 1 is "cargo build"

        let ranked_git = rank_items("zenith", &items);
        assert_eq!(ranked_git.len(), 1);
        assert_eq!(ranked_git[0], 0);
    }
}
