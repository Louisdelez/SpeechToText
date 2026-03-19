use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::Message;
use crate::types::{ChatMessage, MessageRole};
use crate::ui::{icons, theme};

pub fn view<'a>(idx: usize, msg: &'a ChatMessage) -> Element<'a, Message> {
    let time_str = msg.timestamp.format("%H:%M").to_string();

    let copy_btn = button(
        icons::icon(icons::COPY, 12.0).style(theme::muted_text),
    )
    .on_press(Message::CopyText(idx))
    .padding([4, 6])
    .style(theme::copy_button);

    let content = column![
        text(&msg.text).size(14).style(match msg.role {
            MessageRole::User => theme::primary_text,
            MessageRole::Assistant => theme::secondary_text,
        }),
        row![
            text(time_str).size(10).style(theme::muted_text),
            Space::new().width(Length::Fill),
            copy_btn,
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(6);

    let bubble_style = match msg.role {
        MessageRole::User => user_bubble_style as fn(&iced::Theme) -> container::Style,
        MessageRole::Assistant => assistant_bubble_style as fn(&iced::Theme) -> container::Style,
    };

    let bubble = container(content)
        .padding([12, 16])
        .max_width(340)
        .style(bubble_style);

    // User messages aligned right, assistant left
    match msg.role {
        MessageRole::User => {
            container(
                row![Space::new().width(Length::Fill), bubble]
            )
            .width(Length::Fill)
            .into()
        }
        MessageRole::Assistant => {
            container(
                row![bubble, Space::new().width(Length::Fill)]
            )
            .width(Length::Fill)
            .into()
        }
    }
}

fn user_bubble_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.15,
            ..theme::ACCENT_COLOR
        })),
        border: Border {
            radius: 16.0.into(),
            color: Color { a: 0.2, ..theme::ACCENT_COLOR },
            width: 1.0,
        },
        ..Default::default()
    }
}

fn assistant_bubble_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::SURFACE_COLOR)),
        border: Border {
            radius: 16.0.into(),
            color: theme::SURFACE_BORDER,
            width: 1.0,
        },
        ..Default::default()
    }
}
