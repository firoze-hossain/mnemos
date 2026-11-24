//! crates/mnemos-ui/src/lib.rs
//!
//! Mnemos UI: 2D software rasterization, acrylic glass card, popover preview,
//! and pixel-crisp widgets for Mnemos Clipboard Manager.

pub mod draw;
pub mod font;
pub mod geom;
pub mod views;

pub use draw::{OwnedBufferSurface, Surface};
pub use font::{draw_char, draw_text, draw_text_small, Font};
pub use geom::{Color, Point, Rect};
pub use views::{CARD_ROW_HEIGHT, CARD_WIDTH, PREVIEW_HEIGHT, PREVIEW_WIDTH, MnemosRenderer};

#[cfg(test)]
mod tests {
    use super::*;
    use mnemos_core::hub::MnemosHub;

    #[test]
    fn test_rendering_headless_surface() {
        let mut hub = MnemosHub::with_maccy_demo();
        let mut surface = OwnedBufferSurface::new(800, 600);

        // Render full interface with preview
        MnemosRenderer::render(&mut surface, &hub, 50, 50);

        // Check that pixels were written (non-empty)
        let sample = surface.get_pixel(60, 60);
        assert_ne!(sample.to_u32(), 0);

        // Test search filter rasterization
        hub.set_search("cargo");
        MnemosRenderer::render(&mut surface, &hub, 50, 50);

        // Test preferences dialog rasterization
        hub.show_preferences = true;
        MnemosRenderer::render(&mut surface, &hub, 50, 50);
    }
}
