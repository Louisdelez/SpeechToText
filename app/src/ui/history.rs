use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::i18n::Strings;
use crate::types::Conversation;
use crate::ui::{icons, theme};

pub fn view<'a>(conversations: &[Conversation], s: &Strings) -> Element<'a, Message> {
    let title = text(s.history).size(18).style(theme::primary_text);

    let back_btn = button(
        row![
            icons::icon(icons::CHEVRON_LEFT, 16.0).style(theme::secondary_text),
            text(s.back).size(13).style(theme::secondary_text),
        ].spacing(4).align_y(Alignment::Center),
    ).padding([6, 12]).style(theme::pill_button).on_press(Message::ToggleHistory);

    let clear_btn = button(icons::icon(icons::TRASH, 16.0).style(theme::muted_text))
        .padding([6, 8]).style(theme::gear_button).on_press(Message::ClearHistory);

    let header = row![back_btn, Space::new().width(Length::Fill), title, Space::new().width(Length::Fill), clear_btn]
        .align_y(Alignment::Center).width(Length::Fill);

    let non_empty: Vec<_> = conversations.iter().enumerate()
        .filter(|(_, c)| !c.messages.is_empty())
        .collect();

    let content = if non_empty.is_empty() {
        column![
            container(header).padding([20, 8]),
            Space::new().height(Length::Fill),
            container(text(s.no_transcription).size(14).style(theme::muted_text))
                .width(Length::Fill).center_x(Length::Fill),
            Space::new().height(Length::Fill),
        ]
    } else {
        let mut items = column![].spacing(12).padding([4, 0]);

        for (conv_idx, conv) in non_empty.iter().rev() {
            let mode_label = conv.mode.to_string();
            let date_str = conv.created_at.format("%d/%m/%Y %H:%M").to_string();
            let msg_count = conv.messages.len();

            let preview = conv.messages.last().map(|m| {
                if m.text.len() > 100 { format!("{}...", &m.text[..100]) } else { m.text.clone() }
            }).unwrap_or_default();

            let copy_btn = button(icons::icon(icons::COPY, 12.0).style(theme::muted_text))
                .padding([4, 6]).style(theme::copy_button)
                .on_press(Message::CopyHistoryText(conv_idx * 10000 + conv.messages.len() - 1));

            let item = button(
                column![
                    row![
                        text(mode_label).size(12).style(theme::secondary_text),
                        Space::new().width(Length::Fill),
                        text(format!("{msg_count} msg · {date_str}")).size(11).style(theme::muted_text),
                    ].align_y(Alignment::Center),
                    text(preview).size(13).style(theme::primary_text),
                    row![Space::new().width(Length::Fill), copy_btn].align_y(Alignment::Center),
                ].spacing(6),
            ).padding([12, 16]).width(Length::Fill).style(theme::pill_button)
            .on_press(Message::OpenConversation(*conv_idx));

            items = items.push(item);
        }

        column![
            container(header).padding([20, 8]),
            scrollable(items).height(Length::Fill),
        ]
    };

    container(content).width(Length::Fill).height(Length::Fill).into()
}
