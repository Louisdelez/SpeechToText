use iced::widget::{image, text};
use iced::{Element, Font};

pub const LUCIDE_FONT: Font = Font::with_name("lucide");
pub const LUCIDE_BYTES: &[u8] = include_bytes!("../../assets/lucide.ttf");

// Codepoints
pub const SETTINGS: char = '\u{e154}';
pub const HISTORY: char = '\u{e1f5}';
pub const CHEVRON_LEFT: char = '\u{e06e}';
pub const COPY: char = '\u{e09e}';
pub const TRASH: char = '\u{e18e}';
pub const MIC: char = '\u{e118}';
pub const SQUARE: char = '\u{e167}';
pub const UPLOAD: char = '\u{e22f}';
pub const LOADER: char = '\u{e109}';
pub const MENU: char = '\u{e115}';
pub const CHEVRON_DOWN: char = '\u{e06d}';
pub const DOWNLOAD: char = '\u{e099}';
pub const REFRESH: char = '\u{e14d}';

pub fn icon(codepoint: char, size: f32) -> text::Text<'static> {
    text(codepoint.to_string())
        .font(LUCIDE_FONT)
        .size(size)
}

