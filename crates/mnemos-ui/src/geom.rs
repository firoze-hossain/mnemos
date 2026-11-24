//! crates/mnemos-ui/src/geom.rs
//!
//! Geometric primitives, Rect math, and Maccy/NovaOS theme palette for Mnemos.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }

    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn center_x(&self) -> i32 {
        self.x + self.w / 2
    }

    pub fn center_y(&self) -> i32 {
        self.y + self.h / 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn from_u32(val: u32) -> Self {
        Color {
            a: ((val >> 24) & 0xFF) as u8,
            r: ((val >> 16) & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: (val & 0xFF) as u8,
        }
    }

    pub fn blend_over(self, bg: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return bg;
        }
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;
        let r = ((self.r as u32 * alpha + bg.r as u32 * inv_alpha) / 255) as u8;
        let g = ((self.g as u32 * alpha + bg.g as u32 * inv_alpha) / 255) as u8;
        let b = ((self.b as u32 * alpha + bg.b as u32 * inv_alpha) / 255) as u8;
        Color { r, g, b, a: 255 }
    }

    // Maccy translucent dark acrylic glass palette
    pub const MACCY_BG: Color = Color::rgba(30, 33, 42, 245);
    pub const MACCY_CARD_BG: Color = Color::rgba(40, 44, 55, 230);
    pub const MACCY_BORDER: Color = Color::rgba(255, 255, 255, 38);
    pub const MACCY_BORDER_SUBTLE: Color = Color::rgba(255, 255, 255, 20);
    pub const MACCY_DIVIDER: Color = Color::rgba(255, 255, 255, 24);

    // Selection & Accent
    pub const SELECTION_BLUE: Color = Color::rgb(0, 122, 255);       // macOS System Blue
    pub const SELECTION_HOVER: Color = Color::rgba(255, 255, 255, 18);
    pub const ACCENT_BLUE: Color = Color::rgb(56, 189, 248);         // NovaOS Fluent Blue
    pub const ACCENT_PIN: Color = Color::rgb(255, 179, 0);           // Pin Gold/Orange
    pub const ACCENT_DELETE: Color = Color::rgb(248, 113, 113);      // Clear / Delete Red

    // Typography
    pub const TEXT_PRIMARY: Color = Color::rgb(243, 244, 246);
    pub const TEXT_SECONDARY: Color = Color::rgb(156, 163, 175);
    pub const TEXT_MUTED: Color = Color::rgb(107, 114, 128);
    pub const TEXT_SHORTCUT: Color = Color::rgb(148, 163, 184);
    pub const TEXT_WHITE: Color = Color::rgb(255, 255, 255);

    // Popover Preview Colors
    pub const PREVIEW_BG: Color = Color::rgba(26, 29, 36, 250);
    pub const PREVIEW_HEADER: Color = Color::rgba(35, 39, 48, 255);
    pub const PREVIEW_INSET: Color = Color::rgba(18, 20, 25, 255);
    pub const PREVIEW_BADGE_BG: Color = Color::rgba(255, 255, 255, 22);

    // Semantic Status
    pub const STATUS_SUCCESS: Color = Color::rgb(52, 199, 89);
    pub const STATUS_WARNING: Color = Color::rgb(255, 159, 10);
    pub const STATUS_ERROR: Color = Color::rgb(255, 69, 58);
}
