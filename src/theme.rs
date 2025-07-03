// ============================================================================
// em(π)trio MP3 Player — theme.rs
// Author: Tom Papatolis
// Email: tom@tpapatolis.com
// Github: https://github.com/tomgineer/empitrio
// ---------------------------------------------------------------------------
// Description:
// Defines color themes and styling used by the TUI for consistent look & feel.
// ============================================================================

use ratatui::style::Color;

pub struct Theme {
    pub text: Color,
    pub selection_text: Color,
    pub selection_background: Color,
    pub title: Color,
    pub border: Color,
    pub status_text: Color,
    //pub warning_text: Color,
}

impl Theme {
    pub fn xcad() -> Self {
        Self {
            text: Color::Rgb(204, 204, 204),                 // #CCCCCC
            selection_text: Color::Rgb(255, 255, 255),       // #FFFFFF
            selection_background: Color::Rgb(43, 79, 255),   // #2B4FFF
            title: Color::Rgb(241, 241, 241),                // #2B4FFF
            border: Color::Rgb(150, 150, 150),               // #999999
            status_text: Color::Rgb(92, 120, 255),           // #5C78FF
            // warning_text: Color::Rgb(255, 64, 64)         // #FF4040
        }
    }
}

// Color theme "xcad" colors as RGB with hex comments:
//
// background:          Rgb(0, 0, 0)           // #000000
// black:               Rgb(18, 18, 18)        // #121212
// blue:                Rgb(43, 79, 255)       // #2B4FFF
// brightBlack:         Rgb(102, 102, 102)     // #666666
// brightBlue:          Rgb(92, 120, 255)      // #5C78FF
// brightCyan:          Rgb(90, 200, 255)      // #5AC8FF
// brightGreen:         Rgb(144, 90, 255)      // #905AFF
// brightPurple:        Rgb(94, 162, 255)      // #5EA2FF
// brightRed:           Rgb(186, 90, 255)      // #BA5AFF
// brightWhite:         Rgb(255, 255, 255)     // #FFFFFF
// brightYellow:        Rgb(104, 90, 255)      // #685AFF
// cursorColor:         Rgb(255, 255, 255)     // #FFFFFF
// cyan:                Rgb(40, 185, 255)      // #28B9FF
// foreground:          Rgb(241, 241, 241)     // #F1F1F1
// green:               Rgb(113, 41, 255)      // #7129FF
// purple:              Rgb(40, 131, 255)      // #2883FF
// red:                 Rgb(165, 42, 255)      // #A52AFF
// selectionBackground: Rgb(255, 255, 255)     // #FFFFFF
// white:               Rgb(241, 241, 241)     // #F1F1F1
// yellow:              Rgb(61, 42, 255)       // #3D2AFF
