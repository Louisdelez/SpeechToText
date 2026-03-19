use iced::color;
use iced::widget::{button, container, overlay::menu, pick_list, scrollable, text};
use iced::{Background, Border, Color, Shadow, Theme};

// Palette
pub const BG_COLOR: Color = color!(0x0e, 0x0e, 0x10);
pub const SURFACE_COLOR: Color = color!(0x1a, 0x1a, 0x1e);
pub const SURFACE_HOVER: Color = color!(0x24, 0x24, 0x2a);
pub const SURFACE_BORDER: Color = color!(0x2a, 0x2a, 0x30);
pub const ACCENT_COLOR: Color = color!(0x63, 0x66, 0xf1);
pub const RED_COLOR: Color = color!(0xef, 0x44, 0x44);
pub const RED_GLOW: Color = color!(0xef, 0x44, 0x44);
pub const GREEN_COLOR: Color = color!(0x22, 0xc5, 0x5e);
pub const TEXT_PRIMARY: Color = color!(0xf4, 0xf4, 0xf5);
pub const TEXT_SECONDARY: Color = color!(0xa1, 0xa1, 0xaa);
pub const TEXT_MUTED: Color = color!(0x63, 0x63, 0x6b);

// Main background
pub fn main_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(BG_COLOR)),
        ..Default::default()
    }
}

// Header bar - subtle bottom border
pub fn header_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(BG_COLOR)),
        border: Border {
            color: SURFACE_BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

// Transcription bubble
pub fn bubble_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(SURFACE_COLOR)),
        border: Border {
            radius: 16.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}

// Bottom control bar
pub fn controls_container(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(color!(0x12, 0x12, 0x16))),
        border: Border {
            radius: 20.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}

// Record button - big red circle
pub fn record_button_idle(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => Color { a: 0.85, ..RED_COLOR },
        _ => RED_COLOR,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_PRIMARY,
        border: Border {
            radius: 32.0.into(),
            ..Default::default()
        },
        shadow: iced::Shadow {
            color: Color { a: 0.3, ..RED_GLOW },
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

// Record button while recording - pulsing/dimmed
pub fn record_button_active(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => Color { a: 0.7, ..RED_COLOR },
        _ => Color { a: 0.55, ..RED_COLOR },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_PRIMARY,
        border: Border {
            radius: 32.0.into(),
            color: RED_COLOR,
            width: 2.0,
        },
        ..Default::default()
    }
}

// Pill-shaped secondary button (Import, device toggle)
pub fn pill_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => SURFACE_HOVER,
        _ => SURFACE_COLOR,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_SECONDARY,
        border: Border {
            radius: 20.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}

// Copy button - ghost style
pub fn copy_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let (bg, text_color) = match status {
        button::Status::Hovered => (SURFACE_HOVER, TEXT_SECONDARY),
        _ => (Color::TRANSPARENT, TEXT_MUTED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Device toggle - highlighted when GPU
pub fn device_button_gpu(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => Color { a: 0.15, ..GREEN_COLOR },
        _ => Color { a: 0.1, ..GREEN_COLOR },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: GREEN_COLOR,
        border: Border {
            radius: 20.0.into(),
            color: Color { a: 0.3, ..GREEN_COLOR },
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn device_button_cpu(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => SURFACE_HOVER,
        _ => SURFACE_COLOR,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_SECONDARY,
        border: Border {
            radius: 20.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    scrollable::default(theme, status)
}

pub fn primary_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(TEXT_PRIMARY),
    }
}

pub fn secondary_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(TEXT_SECONDARY),
    }
}

pub fn muted_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(TEXT_MUTED),
    }
}

pub fn accent_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(GREEN_COLOR),
    }
}

// Settings: active device button
pub fn device_active_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => Color { a: 0.2, ..ACCENT_COLOR },
        _ => Color { a: 0.12, ..ACCENT_COLOR },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color { a: 1.0, ..ACCENT_COLOR },
        border: Border {
            radius: 12.0.into(),
            color: Color { a: 0.3, ..ACCENT_COLOR },
            width: 1.0,
        },
        ..Default::default()
    }
}

// Settings: inactive device button
pub fn device_inactive_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let bg = match status {
        button::Status::Hovered => SURFACE_HOVER,
        _ => SURFACE_COLOR,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_MUTED,
        border: Border {
            radius: 12.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}

// Pick list - mode selector (main dropdown in header)
pub fn mode_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let _ = theme;
    let (bg, border_color) = match status {
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => (SURFACE_HOVER, ACCENT_COLOR),
        _ => (SURFACE_COLOR, SURFACE_BORDER),
    };
    pick_list::Style {
        text_color: TEXT_PRIMARY,
        placeholder_color: TEXT_MUTED,
        handle_color: TEXT_SECONDARY,
        background: Background::Color(bg),
        border: Border {
            radius: 12.0.into(),
            color: border_color,
            width: 1.0,
        },
    }
}

// Pick list - language selector (smaller, pill-shaped)
pub fn lang_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let _ = theme;
    let (bg, border_color) = match status {
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => (SURFACE_HOVER, Color { a: 0.4, ..ACCENT_COLOR }),
        _ => (color!(0x16, 0x16, 0x1a), SURFACE_BORDER),
    };
    pick_list::Style {
        text_color: TEXT_SECONDARY,
        placeholder_color: TEXT_MUTED,
        handle_color: TEXT_MUTED,
        background: Background::Color(bg),
        border: Border {
            radius: 10.0.into(),
            color: border_color,
            width: 1.0,
        },
    }
}

// Menu dropdown style (the popup that opens)
pub fn dark_menu(theme: &Theme) -> menu::Style {
    let _ = theme;
    menu::Style {
        background: Background::Color(color!(0x1c, 0x1c, 0x22)),
        border: Border {
            radius: 12.0.into(),
            color: SURFACE_BORDER,
            width: 1.0,
        },
        text_color: TEXT_PRIMARY,
        selected_text_color: TEXT_PRIMARY,
        selected_background: Background::Color(Color { a: 0.15, ..ACCENT_COLOR }),
        shadow: Shadow {
            color: Color { a: 0.3, ..Color::BLACK },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
    }
}

// Settings gear button (ghost)
pub fn gear_button(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let (bg, tc) = match status {
        button::Status::Hovered => (SURFACE_HOVER, TEXT_PRIMARY),
        _ => (Color::TRANSPARENT, TEXT_MUTED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: tc,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
