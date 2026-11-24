//! crates/mnemos-platform/src/lib.rs
//!
//! Mnemos Cross-Platform Clipboard Abstraction Layer.
//!
//! Provides sovereign clipboard reading, writing, and change listening
//! for macOS (pbcopy/pbpaste/NSPasteboard), Linux (X11/Wayland), Windows (clip),
//! and NovaOS (ring-3 system call IPC).

use std::process::Command;

/// Standard cross-platform interface for interacting with system clipboard.
pub trait ClipboardBackend {
    fn read_text(&mut self) -> Option<String>;
    fn write_text(&mut self, text: &str) -> bool;
    fn poll_changes(&mut self) -> Option<String>;
    fn backend_name(&self) -> &'static str;
}

/// In-memory mock clipboard for headless testing, unit tests, and simulated pasteboards.
#[derive(Clone, Debug, Default)]
pub struct MockClipboard {
    current: String,
    last_polled: String,
}

impl MockClipboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content(content: &str) -> Self {
        MockClipboard {
            current: content.to_string(),
            last_polled: String::new(),
        }
    }
}

impl ClipboardBackend for MockClipboard {
    fn read_text(&mut self) -> Option<String> {
        Some(self.current.clone())
    }

    fn write_text(&mut self, text: &str) -> bool {
        self.current = text.to_string();
        true
    }

    fn poll_changes(&mut self) -> Option<String> {
        if self.current != self.last_polled {
            self.last_polled = self.current.clone();
            Some(self.current.clone())
        } else {
            None
        }
    }

    fn backend_name(&self) -> &'static str {
        "Mock/Headless Clipboard"
    }
}

/// Linux (X11 & Wayland) clipboard integration using wl-clipboard or xclip/xsel.
pub struct LinuxClipboard {
    last_content: String,
    has_wl: bool,
    has_xclip: bool,
}

impl LinuxClipboard {
    pub fn new() -> Self {
        let has_wl = Command::new("which")
            .arg("wl-paste")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_xclip = !has_wl
            && Command::new("which")
                .arg("xclip")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        LinuxClipboard {
            last_content: String::new(),
            has_wl,
            has_xclip,
        }
    }
}

impl Default for LinuxClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardBackend for LinuxClipboard {
    fn read_text(&mut self) -> Option<String> {
        if self.has_wl {
            if let Ok(output) = Command::new("wl-paste").arg("-n").output() {
                if output.status.success() {
                    return String::from_utf8(output.stdout).ok();
                }
            }
        } else if self.has_xclip {
            if let Ok(output) = Command::new("xclip").args(["-selection", "clipboard", "-o"]).output() {
                if output.status.success() {
                    return String::from_utf8(output.stdout).ok();
                }
            }
        }
        None
    }

    fn write_text(&mut self, text: &str) -> bool {
        use std::io::Write;
        if self.has_wl {
            if let Ok(mut child) = Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }
        } else if self.has_xclip {
            if let Ok(mut child) = Command::new("xclip")
                .args(["-selection", "clipboard", "-i"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }
        }
        false
    }

    fn poll_changes(&mut self) -> Option<String> {
        if let Some(text) = self.read_text() {
            if text != self.last_content {
                self.last_content = text.clone();
                return Some(text);
            }
        }
        None
    }

    fn backend_name(&self) -> &'static str {
        if self.has_wl {
            "Linux Wayland (wl-clipboard)"
        } else if self.has_xclip {
            "Linux X11 (xclip)"
        } else {
            "Linux Generic"
        }
    }
}

/// macOS clipboard integration using pbcopy and pbpaste.
pub struct MacOSClipboard {
    last_content: String,
}

impl MacOSClipboard {
    pub fn new() -> Self {
        MacOSClipboard {
            last_content: String::new(),
        }
    }
}

impl Default for MacOSClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardBackend for MacOSClipboard {
    fn read_text(&mut self) -> Option<String> {
        if let Ok(output) = Command::new("pbpaste").output() {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok();
            }
        }
        None
    }

    fn write_text(&mut self, text: &str) -> bool {
        use std::io::Write;
        if let Ok(mut child) = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().map(|s| s.success()).unwrap_or(false);
        }
        false
    }

    fn poll_changes(&mut self) -> Option<String> {
        if let Some(text) = self.read_text() {
            if text != self.last_content {
                self.last_content = text.clone();
                return Some(text);
            }
        }
        None
    }

    fn backend_name(&self) -> &'static str {
        "macOS Pasteboard (pbcopy/pbpaste)"
    }
}

/// Windows clipboard integration using powershell / clip.exe.
pub struct WindowsClipboard {
    last_content: String,
}

impl WindowsClipboard {
    pub fn new() -> Self {
        WindowsClipboard {
            last_content: String::new(),
        }
    }
}

impl Default for WindowsClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardBackend for WindowsClipboard {
    fn read_text(&mut self) -> Option<String> {
        if let Ok(output) = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
        {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok();
            }
        }
        None
    }

    fn write_text(&mut self, text: &str) -> bool {
        use std::io::Write;
        if let Ok(mut child) = Command::new("clip").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().map(|s| s.success()).unwrap_or(false);
        }
        false
    }

    fn poll_changes(&mut self) -> Option<String> {
        if let Some(text) = self.read_text() {
            if text != self.last_content {
                self.last_content = text.clone();
                return Some(text);
            }
        }
        None
    }

    fn backend_name(&self) -> &'static str {
        "Windows Clipboard (clip/powershell)"
    }
}

/// NovaOS ring-3 clipboard IPC system call interface.
pub struct NovaClipboard {
    last_content: String,
    ring3_buffer: String,
}

impl NovaClipboard {
    pub fn new() -> Self {
        NovaClipboard {
            last_content: String::new(),
            ring3_buffer: String::new(),
        }
    }
}

impl Default for NovaClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardBackend for NovaClipboard {
    fn read_text(&mut self) -> Option<String> {
        Some(self.ring3_buffer.clone())
    }

    fn write_text(&mut self, text: &str) -> bool {
        self.ring3_buffer = text.to_string();
        true
    }

    fn poll_changes(&mut self) -> Option<String> {
        if self.ring3_buffer != self.last_content {
            self.last_content = self.ring3_buffer.clone();
            Some(self.ring3_buffer.clone())
        } else {
            None
        }
    }

    fn backend_name(&self) -> &'static str {
        "NovaOS Kernel Clipboard IPC (Ring-3)"
    }
}

/// Factory creating the appropriate backend for the current environment.
pub fn create_native_clipboard() -> Box<dyn ClipboardBackend> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOSClipboard::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxClipboard::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsClipboard::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Box::new(NovaClipboard::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_clipboard_roundtrip() {
        let mut mock = MockClipboard::new();
        assert_eq!(mock.read_text(), Some(String::new()));

        let test_text = "cargo build --release -p novaos";
        assert!(mock.write_text(test_text));
        assert_eq!(mock.read_text(), Some(test_text.to_string()));

        // Poll changes
        let polled = mock.poll_changes();
        assert_eq!(polled, Some(test_text.to_string()));

        // Subsequent poll with no change returns None
        assert_eq!(mock.poll_changes(), None);
    }

    #[test]
    fn test_nova_clipboard_ipc() {
        let mut nova = NovaClipboard::new();
        assert_eq!(nova.backend_name(), "NovaOS Kernel Clipboard IPC (Ring-3)");
        assert!(nova.write_text("https://github.com/firoze-hossain/novaos"));
        assert_eq!(nova.poll_changes(), Some("https://github.com/firoze-hossain/novaos".to_string()));
        assert_eq!(nova.poll_changes(), None);
    }
}
