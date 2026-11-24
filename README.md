# Mnemos Clipboard Manager 📋

**Mnemos** (*μνήμη* / *Mnemosyne*, Goddess of Memory) is the sovereign, cross-platform pure-Rust clipboard manager and default clipboard application for **NovaOS**. 

Synthesizing the interaction design and visual aesthetic of **macOS Maccy** with sovereign cross-platform backends (macOS, Ubuntu/Linux, Windows, and NovaOS ring-3 kernel IPC), Mnemos provides instant keyboard-driven search, inline color swatches, multiline indicators, flyout preview popovers, and privacy filtering.

---

## 🌟 Key Features

### 1. 🪟 Translucent Acrylic Menu Window
- **Maccy Interaction Geometry**: Compact floating card (~360px wide) featuring dark translucent acrylic glass, rounded corners, and subtle white border.
- **Search Header**: Instant search bar with magnifying glass icon `🔍`, dynamic query input, and quick clear `×` button.
- **Keyboard Navigation**: Up/Down arrow selection, `Enter` to copy & paste, `Esc` or `⌘Q` to dismiss.
- **Numeric Shortcut Badges**: Direct access keys `⇧⌘1` .. `⇧⌘9` (or `1` .. `9`) for immediate one-stroke pasting.

### 2. 🔍 Fuzzy & Substring Search Engine
- **Ranked Matching**: Prioritizes exact matches, prefix matches, and substring matches, boosted by word boundaries (characters after `_`, `-`, `/`, `.`, space).
- **Acronym Matching**: Resolves abbreviations (e.g. `nvs` matches `novaos`, `cb` matches `cargo build`).
- **Dynamic Re-indexing**: Sub-millisecond filtering across thousands of history items.

### 3. 🎨 Inline Metadata & Color Swatches
- **Color Extraction**: Detects 6-digit hex color codes (`#284058`, `284058`) and RGB syntax, rendering an inline filled colored square `■` directly alongside the item.
- **File URLs & Icons**: Recognizes `file://` URLs, formatting them with smart middle-truncation for screenshot and document paths.
- **Multiline Indicators**: Appends a hooked return symbol `↩` for items containing newlines.
- **Deduplication Engine**: Never clutters history with duplicate entries; identical clips increment the copy counter and refresh the timestamp to the top.

### 4. 🪶 Flyout Preview Popover
- **Adjacent Callout Card**: Dynamically floats adjacent to the selected item with a triangular pointer arrow.
- **Detailed Origin Metadata**: Displays the source application name and badge (e.g. `Finder`, `Terminal`, `Figma`, `Safari`).
- **Telemetry Counter**: Tracks first copied time, last copied time, total copy frequency, character count, and byte size.
- **Full Text Viewport**: Scrollable multiline preview of the complete raw clipboard content.
- **Action Hints**: Quick hints for `⌥P` (un)pin, `⌥⌫` delete, and `↵` paste.

### 5. 🛡️ Privacy & Confidential Filtering
- **Password Manager Shield**: Automatically excludes and ignores sensitive items copied from password managers (`1Password`, `Bitwarden`, `KeePassXC`, `Keychain Access`).
- **Credential Sniffing Prevention**: Detects RSA/SSH private keys and confidential tokens to prevent accidental history leakage.
- **Pin Persistence**: Pinned items remain locked at the top of history and are protected against `Clear all` (`⌥⇧⌘⌫`).

---

## 🏗️ Workspace Architecture

```
mnemos/
├── Cargo.toml
├── crates/
│   ├── mnemos-core/        # Pure domain logic: models, deduplication, search, history, config, hub
│   │   ├── src/
│   │   │   ├── item.rs     # ClipItem, ItemKind, hex color detection, middle-truncation
│   │   │   ├── search.rs   # FuzzyMatcher, substring bonus, acronym ranking
│   │   │   ├── history.rs  # ClipHistory, deduplication, pin management, privacy filter
│   │   │   ├── config.rs   # MnemosConfig, popup positions, themes
│   │   │   ├── hub.rs      # MnemosHub reactive state coordinator
│   │   │   └── lib.rs      # Re-exports
│   │   └── Cargo.toml
│   ├── mnemos-ui/          # 2D software rasterizer & Maccy visual components
│   │   ├── src/
│   │   │   ├── geom.rs     # Rect, Point, Color ARGB, acrylic palette
│   │   │   ├── font.rs     # Bitmap typography, Maccy symbols (↩, ■, 📌, ⌘, ⌥, ⇧)
│   │   │   ├── draw.rs     # Surface trait, rounded rects, popover triangles
│   │   │   ├── views.rs    # Main floating card, flyout preview popover, preferences modal
│   │   │   └── lib.rs      # Headless test verification
│   │   └── Cargo.toml
│   ├── mnemos-platform/    # Cross-platform clipboard backend hooks
│   │   ├── src/
│   │   │   └── lib.rs      # macOS (pbcopy), Linux (X11/Wayland), Windows (clip), NovaOS ring-3
│   │   └── Cargo.toml
│   └── mnemos-app/         # Standalone desktop CLI application & QA qualification harness
│       ├── src/
│       │   └── main.rs     # CLI commands, --selftest, --benchmark, --status, --copy, --paste
│       └── Cargo.toml
└── novaos/                 # NovaOS ring-3 bare-metal integration
    ├── ffi.rs              # System call wrappers (SYS_WRITE, SYS_YIELD)
    ├── mnemos.rs           # Pure #[no_std] #[no_main] ring-3 binary
    └── build.sh            # Target cross-compiler and disk fixture generator
```

---

## 🚀 Building & Running

### Cross-Platform Desktop CLI (Linux / macOS / Windows)
```bash
# Run autonomous qualification suite (26 tests across 6 subsystems)
cargo run --release -p mnemos-app -- --selftest

# Benchmark fuzzy search and 2D UI rendering speed
cargo run --release -p mnemos-app -- --benchmark

# Query history with fuzzy matching
cargo run --release -p mnemos-app -- --search "cargo"

# View clipboard telemetry
cargo run --release -p mnemos-app -- --status
```

### NovaOS Bare-Metal Ring-3 Executable
```bash
# Build i686 ELF32 binary and install to NovaOS fixtures
./novaos/build.sh
```

---

## 📜 License
MIT OR Apache-2.0
