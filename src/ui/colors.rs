//! Rose Pine Color Palette Module
//! Based on the official Rose Pine theme: https://rosepinetheme.com/

use ratatui::style::Color;
pub struct RosePine;

impl RosePine {
    pub const BASE: Color = Color::Rgb(25, 23, 36);
    pub const SURFACE: Color = Color::Rgb(31, 29, 46);
    pub const TEXT: Color = Color::Rgb(224, 222, 244);
    pub const SUBTLE: Color = Color::Rgb(144, 140, 170);
    pub const MUTED: Color = Color::Rgb(110, 106, 134);
    pub const LOVE: Color = Color::Rgb(235, 111, 146);
    pub const GOLD: Color = Color::Rgb(246, 193, 119);
    pub const ROSE: Color = Color::Rgb(235, 188, 186);
    pub const FOAM: Color = Color::Rgb(156, 207, 216);
    pub const IRIS: Color = Color::Rgb(196, 167, 231);
    pub const HIGHLIGHT_HIGH: Color = Color::Rgb(82, 79, 103);
    pub const HIGHLIGHT_LOW: Color = Color::Rgb(33, 32, 46);
}
