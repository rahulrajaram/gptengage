//! Color mode detection and palette definitions for idea tree rendering

use crossterm::style::{Attribute, Color, ContentStyle, StyledContent};
use std::env;

/// Supported terminal color modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// 24-bit RGB colors with background overlays
    Truecolor,
    /// 256-color palette (xterm)
    Ansi256,
    /// Basic 8/16 ANSI colors
    Basic,
    /// No colors at all
    None,
}

impl ColorMode {
    /// Parse a `--color` flag value into a ColorMode
    pub fn from_flag(flag: &str) -> Self {
        match flag.to_lowercase().as_str() {
            "truecolor" | "true" | "24bit" => ColorMode::Truecolor,
            "256" | "ansi256" => ColorMode::Ansi256,
            "basic" | "16" => ColorMode::Basic,
            "none" | "off" | "false" => ColorMode::None,
            _ => detect_color_mode(),
        }
    }
}

/// Detect the best color mode based on environment variables
pub fn detect_color_mode() -> ColorMode {
    // $NO_COLOR takes precedence (see https://no-color.org/)
    if env::var("NO_COLOR").is_ok() {
        return ColorMode::None;
    }

    // Check $COLORTERM for truecolor support
    if let Ok(ct) = env::var("COLORTERM") {
        let ct_lower = ct.to_lowercase();
        if ct_lower == "truecolor" || ct_lower == "24bit" {
            return ColorMode::Truecolor;
        }
    }

    // Check $TERM for 256-color support
    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("256color") {
            return ColorMode::Ansi256;
        }
        if term_lower.contains("color") || term_lower.contains("xterm") || term_lower == "screen" {
            return ColorMode::Basic;
        }
    }

    // Default: basic colors if TERM is set, none otherwise
    if env::var("TERM").is_ok() {
        ColorMode::Basic
    } else {
        ColorMode::None
    }
}

/// Color palette for rendering idea trees
#[derive(Debug, Clone)]
pub struct IdeaTreePalette {
    /// L1 header foreground
    pub l1_fg: Color,
    /// L1 header background (Option because only truecolor has backgrounds)
    pub l1_bg: Option<Color>,
    /// L2 header foreground
    pub l2_fg: Color,
    /// L2 header background
    pub l2_bg: Option<Color>,
    /// Root/seed text
    pub root_fg: Color,
    /// Body/description text
    pub body_fg: Option<Color>,
    /// Axis label and metadata
    pub axis_fg: Color,
    /// Tree connectors (dim)
    pub connector_fg: Color,
    /// Whether bold is available
    pub bold: bool,
    /// Whether dim is available
    pub dim: bool,
}

impl IdeaTreePalette {
    /// Build a palette appropriate for the given color mode
    pub fn for_mode(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Truecolor => Self {
                l1_fg: Color::Rgb {
                    r: 86,
                    g: 182,
                    b: 194,
                },
                l1_bg: Some(Color::Rgb {
                    r: 18,
                    g: 32,
                    b: 36,
                }),
                l2_fg: Color::Rgb {
                    r: 229,
                    g: 181,
                    b: 103,
                },
                l2_bg: Some(Color::Rgb {
                    r: 38,
                    g: 30,
                    b: 18,
                }),
                root_fg: Color::White,
                body_fg: Some(Color::Rgb {
                    r: 220,
                    g: 223,
                    b: 228,
                }),
                axis_fg: Color::Rgb {
                    r: 140,
                    g: 140,
                    b: 160,
                },
                connector_fg: Color::Rgb {
                    r: 140,
                    g: 140,
                    b: 160,
                },
                bold: true,
                dim: true,
            },
            ColorMode::Ansi256 => Self {
                l1_fg: Color::AnsiValue(74),
                l1_bg: None,
                l2_fg: Color::AnsiValue(179),
                l2_bg: None,
                root_fg: Color::White,
                body_fg: None,
                axis_fg: Color::AnsiValue(244),
                connector_fg: Color::AnsiValue(244),
                bold: true,
                dim: true,
            },
            ColorMode::Basic => Self {
                l1_fg: Color::Cyan,
                l1_bg: None,
                l2_fg: Color::Yellow,
                l2_bg: None,
                root_fg: Color::White,
                body_fg: None,
                axis_fg: Color::DarkGrey,
                connector_fg: Color::DarkGrey,
                bold: true,
                dim: true,
            },
            ColorMode::None => Self {
                l1_fg: Color::Reset,
                l1_bg: None,
                l2_fg: Color::Reset,
                l2_bg: None,
                root_fg: Color::Reset,
                body_fg: None,
                axis_fg: Color::Reset,
                connector_fg: Color::Reset,
                bold: false,
                dim: false,
            },
        }
    }
}

/// Style a header string for L1 or L2 with appropriate colors
pub fn styled_header(
    text: &str,
    fg: Color,
    bg: Option<Color>,
    bold: bool,
) -> StyledContent<String> {
    let mut style = ContentStyle::new();
    style.foreground_color = Some(fg);
    if let Some(bg_color) = bg {
        style.background_color = Some(bg_color);
    }
    if bold {
        style.attributes.set(Attribute::Bold);
    }
    StyledContent::new(style, text.to_string())
}

/// Style text as dim/muted
pub fn styled_dim(text: &str, fg: Color, dim: bool) -> StyledContent<String> {
    let mut style = ContentStyle::new();
    style.foreground_color = Some(fg);
    if dim {
        style.attributes.set(Attribute::Dim);
    }
    StyledContent::new(style, text.to_string())
}

/// Style axis/metadata label
pub fn styled_axis(text: &str, fg: Color) -> StyledContent<String> {
    let mut style = ContentStyle::new();
    style.foreground_color = Some(fg);
    StyledContent::new(style, text.to_string())
}

/// Style body text
pub fn styled_body(text: &str, fg: Option<Color>) -> StyledContent<String> {
    let mut style = ContentStyle::new();
    if let Some(color) = fg {
        style.foreground_color = Some(color);
    }
    StyledContent::new(style, text.to_string())
}

/// Style connector characters (│, ├, └, ─)
pub fn styled_connector(text: &str, fg: Color, dim: bool) -> StyledContent<String> {
    styled_dim(text, fg, dim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_from_flag() {
        assert_eq!(ColorMode::from_flag("truecolor"), ColorMode::Truecolor);
        assert_eq!(ColorMode::from_flag("24bit"), ColorMode::Truecolor);
        assert_eq!(ColorMode::from_flag("256"), ColorMode::Ansi256);
        assert_eq!(ColorMode::from_flag("ansi256"), ColorMode::Ansi256);
        assert_eq!(ColorMode::from_flag("none"), ColorMode::None);
        assert_eq!(ColorMode::from_flag("off"), ColorMode::None);
        assert_eq!(ColorMode::from_flag("basic"), ColorMode::Basic);
    }

    #[test]
    fn test_palette_truecolor_has_backgrounds() {
        let palette = IdeaTreePalette::for_mode(ColorMode::Truecolor);
        assert!(palette.l1_bg.is_some());
        assert!(palette.l2_bg.is_some());
        assert!(palette.bold);
        assert!(palette.dim);
    }

    #[test]
    fn test_palette_ansi256_no_backgrounds() {
        let palette = IdeaTreePalette::for_mode(ColorMode::Ansi256);
        assert!(palette.l1_bg.is_none());
        assert!(palette.l2_bg.is_none());
    }

    #[test]
    fn test_palette_basic_no_backgrounds() {
        let palette = IdeaTreePalette::for_mode(ColorMode::Basic);
        assert!(palette.l1_bg.is_none());
        assert!(palette.l2_bg.is_none());
    }

    #[test]
    fn test_palette_none_mode() {
        let palette = IdeaTreePalette::for_mode(ColorMode::None);
        assert!(palette.l1_bg.is_none());
        assert!(palette.l2_bg.is_none());
        assert!(!palette.bold);
        assert!(!palette.dim);
        assert_eq!(palette.l1_fg, Color::Reset);
    }

    #[test]
    fn test_styled_header_produces_output() {
        let styled = styled_header("Test", Color::Cyan, None, true);
        let output = format!("{}", styled);
        assert!(output.contains("Test"));
    }

    #[test]
    fn test_styled_dim_produces_output() {
        let styled = styled_dim("dim text", Color::DarkGrey, true);
        let output = format!("{}", styled);
        assert!(output.contains("dim text"));
    }

    #[test]
    fn test_styled_axis_produces_output() {
        let styled = styled_axis("axis", Color::DarkGrey);
        let output = format!("{}", styled);
        assert!(output.contains("axis"));
    }

    #[test]
    fn test_styled_body_with_color() {
        let styled = styled_body("body", Some(Color::White));
        let output = format!("{}", styled);
        assert!(output.contains("body"));
    }

    #[test]
    fn test_styled_body_no_color() {
        let styled = styled_body("body", None);
        let output = format!("{}", styled);
        assert!(output.contains("body"));
    }
}
