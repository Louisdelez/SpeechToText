use std::collections::HashMap;

use iced::widget::{button, column, container, pick_list, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::i18n::Strings;
use crate::models::{self, ModelId};
use crate::types::*;
use crate::ui::{icons, theme};

pub fn view<'a>(
    device: &ComputeDevice,
    switching: bool,
    device_config: &DeviceConfig,
    language: &Language,
    show_lang_dropdown: bool,
    kokoro: &KokoroConfig,
    model_downloads: &HashMap<ModelId, u8>,
    s: &Strings,
) -> Element<'a, Message> {
    let title = text(s.settings).size(18).style(theme::primary_text);

    let back_btn = button(
        row![
            icons::icon(icons::CHEVRON_LEFT, 16.0).style(theme::secondary_text),
            text(s.back).size(13).style(theme::secondary_text),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .style(theme::pill_button)
    .on_press(Message::ToggleSettings);

    let header = row![
        back_btn,
        Space::new().width(Length::Fill),
        title,
        Space::new().width(Length::Fill),
        Space::new().width(80),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Language section
    let lang_label = text(s.language_label).size(14).style(theme::primary_text);
    let lang_desc = text(s.language_desc).size(12).style(theme::muted_text);

    let current_label = text(language.to_string()).size(14).style(theme::primary_text);
    let chevron = icons::icon(icons::CHEVRON_DOWN, 14.0).style(theme::muted_text);

    let lang_btn = button(
        row![
            current_label,
            Space::new().width(Length::Fill),
            chevron,
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([10, 16])
    .width(Length::Fill)
    .style(theme::pill_button)
    .on_press(Message::ToggleLangDropdown);

    let mut lang_content = column![lang_label, lang_desc, Space::new().height(12), lang_btn].spacing(4);

    if show_lang_dropdown {
        for &lang in Language::ALL {
            let label = text(lang.to_string()).size(14).style(theme::primary_text);
            let item_style = if lang == *language {
                theme::device_active_button as fn(&_, _) -> _
            } else {
                theme::device_inactive_button as fn(&_, _) -> _
            };
            let item = button(label)
                .padding([10, 16])
                .width(Length::Fill)
                .style(item_style)
                .on_press(Message::SetLanguage(lang));
            lang_content = lang_content.push(item);
        }
    }

    let lang_section = container(lang_content)
        .padding(20)
        .width(Length::Fill)
        .style(theme::bubble_container);

    // Device section — per-model GPU/CPU/Auto
    let device_title = text(s.acceleration).size(14).style(theme::primary_text);
    let device_desc = text("Choisir le processeur pour chaque modele").size(12).style(theme::muted_text);

    // Helper to build a 3-button row for a DeviceChoice
    let device_row = |current: DeviceChoice, msg_fn: fn(DeviceChoice) -> Message| -> Element<'a, Message> {
        let mut btns = row![].spacing(3).width(140);
        for &choice in DeviceChoice::ALL {
            let active = choice == current;
            let style = if active {
                theme::device_active_button as fn(&_, _) -> _
            } else {
                theme::device_inactive_button as fn(&_, _) -> _
            };
            let mut btn = button(text(choice.to_string()).size(10).center())
                .padding([5, 0])
                .width(Length::Fill)
                .style(style);
            if !active {
                btn = btn.on_press(msg_fn(choice));
            }
            btns = btns.push(btn);
        }
        btns.into()
    };

    let whisper_label = if switching { "Whisper Medium ..." } else { "Whisper Medium" };

    // Build each model row: label on left, buttons on right
    let models_devices: Vec<(&str, DeviceChoice, fn(DeviceChoice) -> Message)> = vec![
        ("Global (defaut)", device_config.global, Message::SetGlobalDevice),
        (whisper_label, device_config.whisper, Message::SetWhisperDevice),
        ("Opus-MT FR → EN", device_config.opus_fr_en, Message::SetOpusFrEnDevice),
        ("Opus-MT EN → FR", device_config.opus_en_fr, Message::SetOpusEnFrDevice),
        ("Qwen 2.5 1.5B", device_config.qwen, Message::SetQwenDevice),
        ("Kokoro TTS", device_config.kokoro, Message::SetKokoroDevice),
    ];

    let mut device_list = column![].spacing(0);
    for (i, (label, current, msg_fn)) in models_devices.iter().enumerate() {
        let label_txt = text(*label).size(12).style(if i == 0 { theme::primary_text } else { theme::secondary_text });
        let model_row = container(
            row![
                label_txt,
                Space::new().width(Length::Fill),
                device_row(*current, *msg_fn),
            ].align_y(Alignment::Center).width(Length::Fill),
        ).padding([8, 0]).width(Length::Fill);

        device_list = device_list.push(model_row);

        // Separator after Global
        if i == 0 {
            let sep = container(Space::new().height(1))
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        r: 1.0, g: 1.0, b: 1.0, a: 0.08,
                    })),
                    ..Default::default()
                });
            device_list = device_list.push(sep);
        }
    }

    let device_section = container(
        column![device_title, device_desc, Space::new().height(10), device_list].spacing(2),
    )
    .padding(20)
    .width(Length::Fill)
    .style(theme::bubble_container);

    // ── Kokoro TTS section ──────────────────────────────────────────
    let kokoro_title = text("Kokoro TTS").size(14).style(theme::primary_text);
    let kokoro_desc = text("Personnalisation de la voix Kokoro").size(12).style(theme::muted_text);

    // Voice picker
    let voice_label = text("Voix").size(12).style(theme::muted_text);
    let voice_picker = pick_list(
        KOKORO_VOICES,
        Some(kokoro.voice),
        Message::SetKokoroVoice,
    )
    .text_size(12.0)
    .padding([8, 12])
    .width(Length::Fill)
    .style(theme::lang_pick_list)
    .menu_style(theme::dark_menu);

    // Speed picker
    let speed_label = text("Vitesse").size(12).style(theme::muted_text);
    let speed_picker = pick_list(
        KokoroSpeed::ALL,
        Some(kokoro.speed),
        Message::SetKokoroSpeed,
    )
    .text_size(12.0)
    .padding([8, 12])
    .width(Length::Fill)
    .style(theme::lang_pick_list)
    .menu_style(theme::dark_menu);

    // Blend toggle
    let blend_label = text("Melange de voix").size(12).style(theme::muted_text);
    let blend_style = if kokoro.blend_enabled {
        theme::device_active_button as fn(&_, _) -> _
    } else {
        theme::device_inactive_button as fn(&_, _) -> _
    };
    let blend_text = if kokoro.blend_enabled { "Active" } else { "Desactive" };
    let blend_btn = button(text(blend_text).size(12).center())
        .padding([8, 16])
        .width(Length::Fill)
        .style(blend_style)
        .on_press(Message::ToggleKokoroBlend);

    let mut kokoro_content = column![
        kokoro_title,
        kokoro_desc,
        Space::new().height(10),
        voice_label,
        voice_picker,
        Space::new().height(6),
        speed_label,
        speed_picker,
        Space::new().height(6),
        blend_label,
        blend_btn,
    ]
    .spacing(2);

    // Blend options (only if enabled)
    if kokoro.blend_enabled {
        let blend_voice_label = text("Deuxieme voix").size(12).style(theme::muted_text);
        let blend_voice_picker = pick_list(
            KOKORO_VOICES,
            Some(kokoro.blend_voice),
            Message::SetKokoroBlendVoice,
        )
        .text_size(12.0)
        .padding([8, 12])
        .width(Length::Fill)
        .style(theme::lang_pick_list)
        .menu_style(theme::dark_menu);

        let ratio_label = text("Ratio (voix 1 / voix 2)").size(12).style(theme::muted_text);
        let ratio_picker = pick_list(
            KokoroBlendRatio::ALL,
            Some(kokoro.blend_ratio),
            Message::SetKokoroBlendRatio,
        )
        .text_size(12.0)
        .padding([8, 12])
        .width(Length::Fill)
        .style(theme::lang_pick_list)
        .menu_style(theme::dark_menu);

        kokoro_content = kokoro_content
            .push(Space::new().height(6))
            .push(blend_voice_label)
            .push(blend_voice_picker)
            .push(Space::new().height(6))
            .push(ratio_label)
            .push(ratio_picker);
    }

    let kokoro_section = container(kokoro_content)
        .padding(20)
        .width(Length::Fill)
        .style(theme::bubble_container);

    // ── Models management section ─────────────────────────────────
    let models_title = text("Gestion des modeles").size(14).style(theme::primary_text);
    let models_desc = text("Telecharger, supprimer ou re-telecharger les modeles").size(12).style(theme::muted_text);

    let mut models_list = column![].spacing(0);

    for (i, &id) in ModelId::ALL.iter().enumerate() {
        let exists = id.exists();
        let downloading = model_downloads.get(&id);

        // Name + status tag
        let status_label = if downloading.is_some() {
            " - telechargement..."
        } else if exists {
            ""
        } else {
            " - absent"
        };
        let name_txt = text(format!("{}{}", id.name(), status_label)).size(13).style(theme::primary_text);

        // Description + size on one line
        let size_str = if let Some(&pct) = downloading {
            format!("{} | {}%", id.description(), pct)
        } else if exists {
            format!("{} | {}", id.description(), models::format_size(id.size_bytes()))
        } else {
            format!("{} | {}", id.description(), id.expected_size())
        };
        let detail_txt = text(size_str).size(10).style(theme::muted_text);

        // Single action button, full width, clear label
        let action_btn: Element<'a, Message> = if downloading.is_some() {
            // Downloading: no action, just wait
            Space::new().width(0).into()
        } else if exists {
            // Two buttons: Retelecharger + Supprimer
            row![
                button(text("Retelecharger").size(10).center())
                    .padding([6, 12])
                    .width(Length::Fill)
                    .style(theme::device_inactive_button)
                    .on_press(Message::RedownloadModel(id)),
                button(text("Supprimer").size(10).center())
                    .padding([6, 12])
                    .width(Length::Fill)
                    .style(theme::device_inactive_button)
                    .on_press(Message::DeleteModel(id)),
            ].spacing(6).width(Length::Fill).into()
        } else {
            button(text("Telecharger").size(10).center())
                .padding([6, 12])
                .width(Length::Fill)
                .style(theme::device_active_button)
                .on_press(Message::DownloadModel(id))
                .into()
        };

        let model_entry = container(
            column![name_txt, detail_txt, Space::new().height(4), action_btn].spacing(2),
        )
        .padding([12, 14])
        .width(Length::Fill);

        models_list = models_list.push(model_entry);

        // Separator (except after last)
        if i < ModelId::ALL.len() - 1 {
            let sep = container(Space::new().height(1))
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        r: 1.0, g: 1.0, b: 1.0, a: 0.06,
                    })),
                    ..Default::default()
                });
            models_list = models_list.push(sep);
        }
    }

    let models_section = container(
        column![models_title, models_desc, Space::new().height(8), models_list].spacing(4),
    )
    .padding(20)
    .width(Length::Fill)
    .style(theme::bubble_container);

    let content = column![
        header,
        Space::new().height(24),
        lang_section,
        Space::new().height(12),
        device_section,
        Space::new().height(12),
        kokoro_section,
        Space::new().height(12),
        models_section,
        Space::new().height(20),
    ]
    .spacing(0)
    .padding([20, 8]);

    iced::widget::scrollable(
        container(content)
            .width(Length::Fill)
            .height(Length::Shrink),
    )
    .height(Length::Fill)
    .into()
}
