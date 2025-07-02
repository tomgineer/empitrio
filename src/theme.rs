// theme.rs

use ratatui::style::Color;

pub struct Theme {
    pub text: Color,
    pub selection_text: Color,
    pub selection_background: Color,
    pub title: Color,
    pub border: Color,
    pub status_text: Color,
}

impl Theme {
    pub fn xcad() -> Self {
        Self {
            text: Color::Rgb(204, 204, 204),                 // #CCCCCC
            selection_text: Color::Rgb(255, 255, 255),       // #FFFFFF
            selection_background: Color::Rgb(43, 79, 255),   // #2B4FFF
            title: Color::Rgb(43, 79, 255),                  // #2B4FFF
            border: Color::Rgb(85, 85, 85),                  // #555555
            status_text: Color::Rgb(92, 120, 255),           // #5C78FF
        }
    }
}
