//! Rose Pine Color Palette Module
//!
//! This module provides the official Rose Pine color palette for RustUI, a beautiful
//! and carefully crafted color scheme designed for comfortable coding and UI design.
//!
//! Based on the official Rose Pine theme: https://rosepinetheme.com/

use ratatui::style::Color;

/// The colors are organized into logical groups:
/// - Base colors for primary backgrounds
/// - Foreground colors for text with different emphasis levels
/// - Accent colors for interactive elements and highlights
/// - Highlight colors for selections and subtle emphasis
///
/// All colors are provided as RGB values compatible with ratatui's Color type.
pub struct RosePine;

impl RosePine {
    /// Primary background color - the darkest base color used for main application backgrounds
    ///
    /// This deep purple-gray serves as the foundation color for the entire interface.
    /// Use this for main panels, application frames, and primary background areas.
    pub const BASE: Color = Color::Rgb(25, 23, 36);

    /// Secondary background color - slightly lighter than BASE for layered elements
    ///
    /// Used for secondary panels, cards, input fields, and elements that need to stand
    /// out slightly from the main background while maintaining the overall dark theme.
    pub const SURFACE: Color = Color::Rgb(31, 29, 46);

    /// High contrast text color - the primary color for all readable text
    ///
    /// This warm off-white provides excellent readability against dark backgrounds
    /// while being easier on the eyes than pure white. Use for primary content,
    /// headings, and any text that needs maximum legibility.
    pub const TEXT: Color = Color::Rgb(224, 222, 244);

    /// Medium contrast text color - for secondary content and less emphasized text
    ///
    /// A muted purple-gray that provides good readability while being less prominent
    /// than the main TEXT color. Perfect for secondary information, captions,
    /// and supplementary content.
    pub const SUBTLE: Color = Color::Rgb(144, 140, 170);

    /// Low contrast text color - for disabled elements and placeholder text
    ///
    /// The most muted text color, used for disabled states, placeholder text,
    /// and content that should be visible but not prominent. Maintains accessibility
    /// while clearly indicating reduced importance.
    pub const MUTED: Color = Color::Rgb(110, 106, 134);

    /// Error and attention color - warm red-pink for warnings and critical actions
    ///
    /// A vibrant but not harsh red-pink that draws attention without being jarring.
    /// Use for error messages, delete actions, warnings, and any content that
    /// requires immediate user attention.
    pub const LOVE: Color = Color::Rgb(235, 111, 146);

    /// Warning and highlight color - warm golden yellow for cautions and emphasis
    ///
    /// A rich golden color that provides warmth and draws attention in a positive way.
    /// Perfect for warning messages, important notifications, and highlighting
    /// significant content that needs user awareness.
    pub const GOLD: Color = Color::Rgb(246, 193, 119);

    /// Accent color - soft pink-beige for gentle highlights and decorative elements
    ///
    /// A subtle, warm accent color that adds visual interest without being overwhelming.
    /// Use for gentle highlights, decorative borders, and elements that need a
    /// soft accent that complements the overall warm tone.
    pub const ROSE: Color = Color::Rgb(235, 188, 186);

    /// Information and interactive color - calming blue-cyan for links and info
    ///
    /// A soothing blue-cyan that conveys trust and information. Ideal for links,
    /// informational messages, interactive elements, and any content that suggests
    /// helpfulness or additional information availability.
    pub const FOAM: Color = Color::Rgb(156, 207, 216);

    /// Special accent color - purple for unique elements and current selections
    ///
    /// A distinctive purple that stands out beautifully against the Rose Pine palette.
    /// Perfect for current selections, active states, special badges, and elements
    /// that need to be clearly identified as current or special.
    pub const IRIS: Color = Color::Rgb(196, 167, 231);

    /// Border and divider color - medium contrast for visual separation
    ///
    /// A balanced color that provides clear visual separation without being harsh.
    /// Use for borders around elements, dividers between sections, and any lines
    /// that need to create visual structure in the interface.
    pub const HIGHLIGHT_HIGH: Color = Color::Rgb(82, 79, 103);

    /// Selection background color - subtle highlight for selected items
    ///
    /// A very subtle background color that indicates selection or hover states.
    /// This color is designed to be noticeable but not distracting, perfect for
    /// highlighting the current item in lists or menus.
    pub const HIGHLIGHT_LOW: Color = Color::Rgb(33, 32, 46);
}
