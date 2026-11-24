//! crates/mnemos-ui/src/draw.rs
//!
//! Software rasterizer for Mnemos: translucent acrylic glass background,
//! rounded rectangles, callout triangles for popovers, badges, and inline swatches.

use crate::geom::{Color, Rect};

pub trait Surface {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn get_pixel(&self, x: i32, y: i32) -> Color;

    fn fill_rect(&mut self, r: Rect, color: Color) {
        let x0 = r.x.max(0);
        let y0 = r.y.max(0);
        let x1 = (r.x + r.w).min(self.width());
        let y1 = (r.y + r.h).min(self.height());
        for y in y0..y1 {
            for x in x0..x1 {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn draw_rect(&mut self, r: Rect, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        self.draw_line(r.x, r.y, r.x + r.w - 1, r.y, color);
        self.draw_line(r.x, r.y + r.h - 1, r.x + r.w - 1, r.y + r.h - 1, color);
        self.draw_line(r.x, r.y, r.x, r.y + r.h - 1, color);
        self.draw_line(r.x + r.w - 1, r.y, r.x + r.w - 1, r.y + r.h - 1, color);
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let mut x = r;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.put_pixel(cx + x, cy + y, color);
            self.put_pixel(cx + y, cy + x, color);
            self.put_pixel(cx - y, cy + x, color);
            self.put_pixel(cx - x, cy + y, color);
            self.put_pixel(cx - x, cy - y, color);
            self.put_pixel(cx - y, cy - x, color);
            self.put_pixel(cx + y, cy - x, color);
            self.put_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    self.put_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    /// Fills a rounded rectangle with corner radius `radius`.
    fn fill_rounded_rect(&mut self, r: Rect, radius: i32, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        let rad = radius.min(r.w / 2).min(r.h / 2);
        if rad <= 0 {
            self.fill_rect(r, color);
            return;
        }

        // Center cross
        self.fill_rect(Rect::new(r.x + rad, r.y, r.w - 2 * rad, r.h), color);
        self.fill_rect(Rect::new(r.x, r.y + rad, rad, r.h - 2 * rad), color);
        self.fill_rect(Rect::new(r.x + r.w - rad, r.y + rad, rad, r.h - 2 * rad), color);

        // Four corners
        self.fill_circle(r.x + rad, r.y + rad, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + rad, rad, color);
        self.fill_circle(r.x + rad, r.y + r.h - rad - 1, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + r.h - rad - 1, rad, color);
    }

    /// Outlines a rounded rectangle with corner radius `radius`.
    fn draw_rounded_rect(&mut self, r: Rect, radius: i32, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        let rad = radius.min(r.w / 2).min(r.h / 2);
        if rad <= 0 {
            self.draw_rect(r, color);
            return;
        }

        // Horizontal edges
        self.draw_line(r.x + rad, r.y, r.x + r.w - rad - 1, r.y, color);
        self.draw_line(r.x + rad, r.y + r.h - 1, r.x + r.w - rad - 1, r.y + r.h - 1, color);

        // Vertical edges
        self.draw_line(r.x, r.y + rad, r.x, r.y + r.h - rad - 1, color);
        self.draw_line(r.x + r.w - 1, r.y + rad, r.x + r.w - 1, r.y + r.h - rad - 1, color);

        // Corner arcs
        let mut x = rad;
        let mut y = 0;
        let mut err = 0;

        let tl = (r.x + rad, r.y + rad);
        let tr = (r.x + r.w - rad - 1, r.y + rad);
        let bl = (r.x + rad, r.y + r.h - rad - 1);
        let br = (r.x + r.w - rad - 1, r.y + r.h - rad - 1);

        while x >= y {
            // Top Left
            self.put_pixel(tl.0 - x, tl.1 - y, color);
            self.put_pixel(tl.0 - y, tl.1 - x, color);

            // Top Right
            self.put_pixel(tr.0 + x, tr.1 - y, color);
            self.put_pixel(tr.0 + y, tr.1 - x, color);

            // Bottom Left
            self.put_pixel(bl.0 - x, bl.1 + y, color);
            self.put_pixel(bl.0 - y, bl.1 + x, color);

            // Bottom Right
            self.put_pixel(br.0 + x, br.1 + y, color);
            self.put_pixel(br.0 + y, br.1 + x, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Renders a magnifying glass icon for the search bar.
    fn draw_search_icon(&mut self, cx: i32, cy: i32, color: Color) {
        self.draw_circle(cx - 2, cy - 2, 4, color);
        self.draw_line(cx + 1, cy + 1, cx + 5, cy + 5, color);
        self.draw_line(cx + 2, cy + 1, cx + 5, cy + 4, color);
    }

    /// Renders a triangle pointer callout for popovers.
    /// `dir`: 0 = pointing left, 1 = pointing right, 2 = pointing up, 3 = pointing down
    fn draw_popover_triangle(&mut self, tip_x: i32, tip_y: i32, size: i32, dir: u8, bg: Color, border: Color) {
        match dir {
            0 => {
                // Pointing Left: tip at (tip_x, tip_y), base at x = tip_x + size
                for dx in 0..=size {
                    let half = (dx * size) / (size * 2).max(1);
                    for dy in -half..=half {
                        self.put_pixel(tip_x + dx, tip_y + dy, bg);
                    }
                }
                self.draw_line(tip_x, tip_y, tip_x + size, tip_y - size / 2, border);
                self.draw_line(tip_x, tip_y, tip_x + size, tip_y + size / 2, border);
            }
            1 => {
                // Pointing Right: tip at (tip_x, tip_y), base at x = tip_x - size
                for dx in 0..=size {
                    let half = (dx * size) / (size * 2).max(1);
                    for dy in -half..=half {
                        self.put_pixel(tip_x - dx, tip_y + dy, bg);
                    }
                }
                self.draw_line(tip_x, tip_y, tip_x - size, tip_y - size / 2, border);
                self.draw_line(tip_x, tip_y, tip_x - size, tip_y + size / 2, border);
            }
            _ => {}
        }
    }
}

/// Owned pixel buffer for headless testing, tests, and offscreen compositing.
#[derive(Clone)]
pub struct OwnedBufferSurface {
    pub pixels: Vec<u32>,
    pub width: i32,
    pub height: i32,
}

impl OwnedBufferSurface {
    pub fn new(w: i32, h: i32) -> Self {
        OwnedBufferSurface {
            pixels: vec![Color::MACCY_BG.to_u32(); (w * h) as usize],
            width: w,
            height: h,
        }
    }
}

impl Surface for OwnedBufferSurface {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else if color.a > 0 {
                let bg = Color::from_u32(self.pixels[idx]);
                self.pixels[idx] = color.blend_over(bg).to_u32();
            }
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Color::rgba(0, 0, 0, 0);
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            Color::from_u32(self.pixels[idx])
        } else {
            Color::rgba(0, 0, 0, 0)
        }
    }
}
