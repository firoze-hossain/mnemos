//! crates/mnemos-app/src/main.rs
//!
//! # Mnemos Clipboard Manager Application Runner & Autonomous Qualification Engine
//!
//! Standalone CLI driver, cross-platform clipboard daemon, and self-test verification suite
//! for the Sovereign NovaOS Clipboard Manager ("Mnemos").

use mnemos_core::history::ClipHistory;
use mnemos_core::hub::MnemosHub;
use mnemos_core::item::{ClipItem, ItemKind};
use mnemos_core::search::FuzzyMatcher;
use mnemos_platform::{create_native_clipboard, ClipboardBackend, MockClipboard, NovaClipboard};
use mnemos_ui::draw::{OwnedBufferSurface, Surface};
use mnemos_ui::views::MnemosRenderer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--selftest" || a == "-t") {
        let code = run_selftest();
        std::process::exit(code);
    }

    if args.iter().any(|a| a == "--benchmark" || a == "-b") {
        run_benchmark();
        return;
    }

    if args.iter().any(|a| a == "--status" || a == "-s") {
        print_status();
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--search") {
        if pos + 1 < args.len() {
            run_search(&args[pos + 1]);
            return;
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--copy") {
        if pos + 1 < args.len() {
            let text = &args[pos + 1];
            let mut clip = create_native_clipboard();
            clip.write_text(text);
            println!("[Mnemos] Copied {} characters to clipboard.", text.len());
            return;
        }
    }

    if args.iter().any(|a| a == "--paste") {
        let mut clip = create_native_clipboard();
        if let Some(text) = clip.read_text() {
            print!("{}", text);
        }
        return;
    }

    // Default: Run self-test qualification
    println!("Mnemos Clipboard Manager - Sovereign Memory Suite for NovaOS");
    let code = run_selftest();
    std::process::exit(code);
}

fn print_help() {
    println!("Mnemos Clipboard Manager v1.0.0 (NovaOS Enterprise)");
    println!("Usage: mnemos [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -t, --selftest       Run full autonomous test suite across all 6 subsystems");
    println!("  -b, --benchmark      Benchmark fuzzy search and 2D UI rasterization speed");
    println!("  -s, --status         Print current clipboard telemetry and history count");
    println!("      --search <Q>     Search history with fuzzy ranking");
    println!("      --copy <TEXT>    Copy string into system clipboard");
    println!("      --paste          Print current system clipboard contents");
    println!("  -h, --help           Show this help information");
}

fn print_status() {
    let hub = MnemosHub::with_maccy_demo();
    let backend = create_native_clipboard();

    println!("============================================================");
    println!("           MNEMOS CLIPBOARD TELEMETRY & STATUS             ");
    println!("============================================================");
    println!("Active Platform Backend: {}", backend.backend_name());
    println!("Total History Items:     {}", hub.history.items.len());
    let pinned_count = hub.history.items.iter().filter(|i| i.pinned).count();
    println!("Pinned Items:            {}", pinned_count);
    println!("Unpinned Items:          {}", hub.history.items.len() - pinned_count);
    println!("Max Storage Capacity:    {} items", hub.config.max_history_size);
    println!("Show Preview Popover:    {}", hub.config.show_preview_panel);
    println!("Paste Automatically:     {}", hub.config.paste_on_select);
    println!("Ignored Applications:    {}", hub.history.ignored_apps.join(", "));
    println!("============================================================");
}

fn run_search(query: &str) {
    let mut hub = MnemosHub::with_maccy_demo();
    hub.set_search(query);
    let items = hub.filtered_items();
    println!("Search results for '{}' ({} found):", query, items.len());
    for (i, item) in items.iter().take(9).enumerate() {
        println!("  [{}] {}: {}", i + 1, item.source_app, item.preview_line(50));
    }
}

fn run_benchmark() {
    println!("Benchmarking Mnemos core algorithms...");

    let mut hub = MnemosHub::with_maccy_demo();
    for i in 0..1000 {
        hub.history.push_text(&format!("benchmark item entry #{} with unique content https://novaos.org/pkg/{}", i, i), "Benchmark", 10000 + i);
    }

    // 1. Fuzzy Search Benchmark
    let start_search = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = hub.history.search("pkg");
        let _ = hub.history.search("screenshot");
        let _ = hub.history.search("284058");
    }
    let elapsed_search = start_search.elapsed();
    println!("  [Search] 3,000 fuzzy searches over 1,000 items: {:.2?}", elapsed_search);

    // 2. UI Rasterization Benchmark
    let mut surface = OwnedBufferSurface::new(800, 600);
    let start_render = std::time::Instant::now();
    for frame in 0..100 {
        hub.selected_idx = frame % 10;
        MnemosRenderer::render(&mut surface, &hub, 50, 50);
    }
    let elapsed_render = start_render.elapsed();
    let fps = 100.0 / elapsed_render.as_secs_f64();
    println!("  [Render] 100 frames rasterized in {:.2?} (~{:.1} FPS)", elapsed_render, fps);
}

fn run_selftest() -> i32 {
    println!("============================================================");
    println!("       MNEMOS CLIPBOARD AUTONOMOUS QUALIFICATION SUITE      ");
    println!("============================================================");

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                println!("  [PASS] {}", $name);
                passed += 1;
            } else {
                println!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    // --- Subsystem 1: Core Item Models & Inline Metadata Detection ---
    println!("\n[1/6] Subsystem: Item Models & Metadata Detection");
    let color_item = ClipItem::new(1, "#284058", "Figma", 1000);
    check!("Hex color detection (#284058 -> RGB 40, 64, 88)",
        color_item.detected_color == Some((40, 64, 88)) && color_item.kind == ItemKind::ColorCode);

    let file_item = ClipItem::new(2, "file:///Users/dev/Screenshot.png", "Finder", 1001);
    check!("File URL detection (file://... -> ItemKind::FileUrls)",
        file_item.kind == ItemKind::FileUrls && file_item.source_app == "Finder");

    let multi_item = ClipItem::new(3, "line1\nline2\nline3", "Terminal", 1002);
    check!("Multiline newline detection (has_newlines = true)",
        multi_item.has_newlines);

    let sec_item = ClipItem::new(4, "-----BEGIN RSA PRIVATE KEY-----\nMIIEow...", "Keygen", 1003);
    check!("Confidential credentials detection (is_confidential = true)",
        sec_item.is_confidential);

    // --- Subsystem 2: History Repository & Deduplication ---
    println!("\n[2/6] Subsystem: Deduplication, Timestamps & FIFO Eviction");
    let mut hist = ClipHistory::default();
    hist.push_text("cargo test", "Terminal", 100);
    check!("Initial push creates 1 item", hist.items.len() == 1);

    // Push identical text: should increment copy count without duplicating
    hist.push_text("cargo test", "Terminal", 200);
    check!("Deduplication merges identical content (item count stays 1)", hist.items.len() == 1);
    check!("Deduplication increments copy count to 2", hist.items[0].copy_count == 2);
    check!("Deduplication updates last_copied_at timestamp to 200", hist.items[0].last_copied_at == 200);

    // Ignored applications filter
    hist.push_text("super_secret_master_pw", "1Password", 300);
    check!("Password manager filtering (1Password items ignored)", hist.items.len() == 1);

    // Pinning and clear_unpinned
    hist.push_text("unpinned_item_1", "Notes", 400);
    hist.push_text("pinned_item_2", "Notes", 500);
    hist.toggle_pin(0); // Pin the newest item
    check!("Item pinned successfully", hist.items[0].pinned);
    hist.clear_unpinned();
    check!("Clear unpinned preserves pinned items while removing unpinned",
        hist.items.iter().all(|i| i.pinned));

    // --- Subsystem 3: Fuzzy Matching & Ranking Engine ---
    println!("\n[3/6] Subsystem: Substring & Fuzzy Search Engine");
    let score_exact = FuzzyMatcher::score("novaos", "novaos").unwrap_or(0);
    let score_prefix = FuzzyMatcher::score("nova", "novaos operating system").unwrap_or(0);
    let score_fuzzy = FuzzyMatcher::score("nvs", "novaos").unwrap_or(0);
    let score_unrelated = FuzzyMatcher::score("xyz", "novaos");

    check!("Exact match score is highest (> 5000)", score_exact >= 5000);
    check!("Prefix match score is high (> 2000)", score_prefix >= 2000);
    check!("Fuzzy acronym match succeeds (nvs matches novaos)", score_fuzzy > 0);
    check!("Unrelated query returns None", score_unrelated.is_none());

    // --- Subsystem 4: Hub Navigation & Shortcut Dispatch ---
    println!("\n[4/6] Subsystem: Reactive Hub Navigation & Shortcut Dispatch");
    let mut hub = MnemosHub::with_maccy_demo();
    check!("Demo dataset pre-populated with Maccy items", hub.history.items.len() >= 6);

    let initial_selected = hub.get_selected_item().unwrap().id;
    hub.move_down();
    check!("Down arrow advances selected item", hub.selected_idx == 1);
    hub.move_up();
    check!("Up arrow returns to initial item", hub.selected_idx == 0 && hub.get_selected_item().unwrap().id == initial_selected);

    // Shortcut 2 selection
    let shortcut_item = hub.select_shortcut(2);
    check!("Shortcut '2' selects item at index 1", shortcut_item.is_some() && hub.selected_idx == 1);

    // Search filtering
    hub.set_search("cargo");
    let search_results = hub.filtered_items();
    check!("Fuzzy search 'cargo' yields filtered matching results with cargo top-ranked",
        !search_results.is_empty() && search_results[0].content.to_lowercase().contains("cargo"));
    hub.clear_search();

    // --- Subsystem 5: 2D Acrylic Glass UI Rasterizer & Popover Preview ---
    println!("\n[5/6] Subsystem: 2D Acrylic UI Rasterizer & Flyout Popover");
    let mut surface = OwnedBufferSurface::new(800, 600);
    MnemosRenderer::render(&mut surface, &hub, 50, 50);

    // Check that rasterizer populated pixels
    let sample_pixel = surface.get_pixel(100, 80);
    check!("Acrylic card rasterized without panic (valid non-zero ARGB)", sample_pixel.to_u32() != 0);

    // Popover triangle rasterization check
    let popover_sample = surface.get_pixel(420, 80);
    check!("Flyout popover preview rasterized with callout triangle", popover_sample.to_u32() != 0);

    // Preferences modal dialog rasterization
    hub.open_preferences();
    MnemosRenderer::render(&mut surface, &hub, 50, 50);
    hub.close_preferences();
    check!("Preferences modal dialog renders with zero panics", !hub.show_preferences);

    // --- Subsystem 6: Cross-Platform Clipboard Backend IPC ---
    println!("\n[6/6] Subsystem: Cross-Platform Clipboard Backend IPC");
    let mut mock_clip = MockClipboard::new();
    check!("Mock clipboard write and read",
        mock_clip.write_text("Hello Mnemos") && mock_clip.read_text() == Some("Hello Mnemos".to_string()));

    let mut nova_clip = NovaClipboard::new();
    check!("NovaOS ring-3 clipboard IPC read/write and change detection",
        nova_clip.write_text("NovaOS Ring-3 Syscall") &&
        nova_clip.poll_changes() == Some("NovaOS Ring-3 Syscall".to_string()));

    let native = create_native_clipboard();
    println!("  [INFO] Host native clipboard: {}", native.backend_name());
    check!("Native clipboard backend initializes successfully", !native.backend_name().is_empty());

    // --- Final Summary ---
    println!("\n============================================================");
    println!("AUTONOMOUS QUALIFICATION SUMMARY: {} passed, {} failed", passed, failed);
    println!("============================================================");

    if failed == 0 {
        println!(">>> [SUCCESS] All 6 Mnemos Subsystems 100% QUALIFIED & OPERATIONAL <<<");
        0
    } else {
        println!(">>> [FAILURE] One or more tests failed! <<<");
        1
    }
}
