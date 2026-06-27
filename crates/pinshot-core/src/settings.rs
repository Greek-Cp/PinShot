//! The persisted settings schema (FR-039–FR-045).
//!
//! Settings are stored as a local, human-readable `Settings.toml` (the shell
//! owns the file I/O and side effects); this module defines the schema,
//! documented defaults, validation, and TOML (de)serialisation — all pure and
//! unit-tested. `#[serde(default)]` everywhere means a missing or partial file
//! loads with defaults rather than failing (the shell falls back to
//! [`Settings::default`] on a corrupt file).

use serde::{Deserialize, Serialize};

/// UI theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

/// UI / OCR language (OCR itself is deferred; language seeds future features).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Id,
}

/// Capture mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureMode {
    #[default]
    Region,
    Window,
    FullScreen,
}

/// Output image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Png,
    Jpg,
    Webp,
}

/// What the Copy action places on the clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ClipboardBehavior {
    #[default]
    Image,
    ImageAndFile,
    FileOnly,
}

/// Scope a hotkey applies in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyScope {
    #[default]
    Global,
    Editor,
}

/// A customisable shortcut binding (FR-041). `chord` is a portable text form
/// (e.g. `"Cmd+Shift+A"`); the shell parses/registers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hotkey {
    pub action: String,
    pub chord: String,
    #[serde(default)]
    pub scope: HotkeyScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    pub launch_at_login: bool,
    pub check_updates: bool,
    pub theme: Theme,
    pub language: Lang,
}

impl Default for General {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            check_updates: false, // opt-in / off (Principle I)
            theme: Theme::System,
            language: Lang::En,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Capture {
    pub mode: CaptureMode,
    pub delay_secs: u8,
    pub include_cursor: bool,
    pub include_shadow: bool,
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            mode: CaptureMode::Region,
            delay_secs: 0,
            include_cursor: false,
            include_shadow: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnnotationDefaults {
    /// `#RRGGBB`.
    pub stroke_color: String,
    /// `#RRGGBB` or empty for no fill.
    pub fill_color: String,
    pub font: String,
    pub font_size: u32,
    pub arrow_size: u32,
    /// 0.0–1.0.
    pub highlighter_opacity: f32,
    pub blur_strength: u32,
    pub pixelate_size: u32,
}

impl Default for AnnotationDefaults {
    fn default() -> Self {
        Self {
            stroke_color: "#EF4444".to_string(),
            fill_color: String::new(),
            font: "PinShot 5x7".to_string(),
            font_size: 16,
            arrow_size: 4,
            highlighter_opacity: 0.4,
            blur_strength: 8,
            pixelate_size: 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportProfile {
    pub format: ImageFormat,
    pub filename_pattern: String,
    /// Quality for JPG/WebP (ignored for PNG), 1–100.
    pub compression: u8,
    pub clipboard: ClipboardBehavior,
}

impl Default for ExportProfile {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            filename_pattern: "PinShot_{date}_{time}".to_string(),
            compression: 90,
            clipboard: ClipboardBehavior::Image,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Advanced {
    pub developer_mode: bool,
}

/// The whole persisted configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub general: General,
    pub capture: Capture,
    pub annotation: AnnotationDefaults,
    pub export: ExportProfile,
    pub advanced: Advanced,
    pub hotkeys: Vec<Hotkey>,
}

impl Settings {
    /// Validates value ranges, returning a list of problems (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if !(0.0..=1.0).contains(&self.annotation.highlighter_opacity) {
            errs.push("annotation.highlighter_opacity must be between 0 and 1".into());
        }
        if !is_hex(&self.annotation.stroke_color) {
            errs.push("annotation.stroke_color must be #RRGGBB".into());
        }
        if !self.annotation.fill_color.is_empty() && !is_hex(&self.annotation.fill_color) {
            errs.push("annotation.fill_color must be empty or #RRGGBB".into());
        }
        if self.export.compression < 1 || self.export.compression > 100 {
            errs.push("export.compression must be between 1 and 100".into());
        }
        if !self.export.filename_pattern.contains("{date}")
            && !self.export.filename_pattern.contains("{time}")
        {
            errs.push("export.filename_pattern should include {date} or {time}".into());
        }
        errs
    }

    /// Parses settings from a TOML string. Missing fields take defaults; an
    /// invalid document is an error (the shell then falls back to defaults).
    pub fn from_toml(s: &str) -> Result<Settings, String> {
        toml::from_str(s).map_err(|e| e.to_string())
    }

    /// Serialises settings to a pretty TOML string.
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

fn is_hex(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_privacy_safe_and_valid() {
        let s = Settings::default();
        assert!(!s.general.check_updates, "update check must default OFF");
        assert!(!s.general.launch_at_login);
        assert_eq!(s.general.theme, Theme::System);
        assert_eq!(s.export.format, ImageFormat::Png);
        assert!(
            s.validate().is_empty(),
            "defaults must validate: {:?}",
            s.validate()
        );
    }

    #[test]
    fn toml_round_trips() {
        let mut s = Settings::default();
        s.general.theme = Theme::Dark;
        s.capture.delay_secs = 3;
        s.hotkeys.push(Hotkey {
            action: "captureRegion".into(),
            chord: "Cmd+Shift+A".into(),
            scope: HotkeyScope::Global,
        });
        let toml = s.to_toml().expect("serialises");
        let back = Settings::from_toml(&toml).expect("parses");
        assert_eq!(s, back);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        // Only a theme given; everything else must default.
        let back = Settings::from_toml("[general]\ntheme = \"dark\"\n").expect("parses");
        assert_eq!(back.general.theme, Theme::Dark);
        assert_eq!(back.export.format, ImageFormat::Png);
        assert_eq!(back.capture.mode, CaptureMode::Region);
    }

    #[test]
    fn invalid_toml_is_an_error() {
        assert!(Settings::from_toml("this is not = = toml").is_err());
    }

    #[test]
    fn validate_flags_bad_ranges() {
        let mut s = Settings::default();
        s.annotation.highlighter_opacity = 2.0;
        s.annotation.stroke_color = "red".into();
        s.export.compression = 0;
        let errs = s.validate();
        assert_eq!(errs.len(), 3, "got {errs:?}");
    }
}
