use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::Message;
use crate::i18n::Strings;
use crate::types::RecordingState;
use crate::ui::{icons, theme};

fn spinner<'a, M: 'a>(tick: u32) -> Element<'a, M> {
    // 8 bars arranged in a row, each with varying opacity based on tick
    let num_bars = 8;
    let mut bars = row![].spacing(3).align_y(Alignment::Center);

    for i in 0..num_bars {
        // Each bar has a phase offset, creating a "wave" animation
        let phase = ((tick as i32 - i as i32).rem_euclid(num_bars as i32)) as f32;
        let alpha = 0.15 + 0.85 * (1.0 - phase / num_bars as f32);
        let height = 8.0 + 12.0 * (1.0 - phase / num_bars as f32);

        let bar: Element<'a, M> = container(text("").size(1))
            .width(3)
            .height(height)
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: alpha,
                })),
                border: Border {
                    radius: 1.5.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();
        bars = bars.push(bar);
    }

    container(bars)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn view<'a>(
    state: &RecordingState,
    elapsed_secs: f32,
    input_value: &str,
    audio_levels: &[f32],
    spinner_tick: u32,
    s: &Strings,
) -> Element<'a, Message> {
    // Record button
    let rec_content: iced::Element<'a, Message> = match state {
        RecordingState::Idle => icons::icon(icons::MIC, 20.0)
            .center()
            .style(theme::primary_text)
            .into(),
        RecordingState::Recording => icons::icon(icons::SQUARE, 18.0)
            .center()
            .style(theme::primary_text)
            .into(),
        RecordingState::Processing | RecordingState::Translating => {
            spinner(spinner_tick)
        }
    };

    let rec_style = match state {
        RecordingState::Recording => theme::record_button_active as fn(&_, _) -> _,
        _ => theme::record_button_idle as fn(&_, _) -> _,
    };

    let mut rec_btn = button(rec_content)
        .width(56)
        .height(56)
        .style(rec_style);

    if *state != RecordingState::Processing && *state != RecordingState::Translating {
        rec_btn = rec_btn.on_press(Message::ToggleRecording);
    }

    // Import button
    let import_btn = button(
        icons::icon(icons::UPLOAD, 14.0).style(theme::secondary_text),
    )
    .padding([8, 10])
    .style(theme::pill_button)
    .on_press(Message::ImportFile);

    // Timer
    let timer = if *state == RecordingState::Recording {
        let mins = (elapsed_secs as u32) / 60;
        let secs = (elapsed_secs as u32) % 60;
        text(format!("{mins:02}:{secs:02}"))
            .size(14)
            .style(theme::primary_text)
    } else {
        text("").size(14)
    };

    // Text input
    let mut input = text_input("Ecrire un texte...", input_value)
        .on_input(Message::TextInputChanged)
        .padding([8, 12])
        .size(13.0);

    if !input_value.trim().is_empty()
        && *state == RecordingState::Idle
    {
        input = input.on_submit(Message::SubmitTextInput);
    }

    // Send button
    let send_btn = if !input_value.trim().is_empty() && *state == RecordingState::Idle {
        button(
            icons::icon(icons::UPLOAD, 16.0).style(theme::primary_text),
        )
        .padding([8, 10])
        .style(theme::record_button_idle)
        .on_press(Message::SubmitTextInput)
    } else {
        button(
            icons::icon(icons::UPLOAD, 16.0).style(theme::muted_text),
        )
        .padding([8, 10])
        .style(theme::pill_button)
    };

    let input_row = row![input, send_btn]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // Waveform visualization (only during recording)
    let waveform: Option<Element<'a, Message>> = if *state == RecordingState::Recording && !audio_levels.is_empty() {
        let bars: Vec<Element<'a, Message>> = audio_levels
            .iter()
            .map(|&level| {
                let height = (level * 28.0).max(3.0);
                container(text("").size(1))
                    .width(4)
                    .height(height)
                    .style(move |_: &iced::Theme| iced::widget::container::Style {
                        background: Some(Background::Color(Color {
                            a: 0.4 + level * 0.6,
                            ..theme::RED_COLOR
                        })),
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
            })
            .collect();

        let mut waveform_row = row![].spacing(2).align_y(Alignment::Center);
        for bar in bars {
            waveform_row = waveform_row.push(bar);
        }

        Some(
            container(waveform_row)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding([4, 0])
                .into()
        )
    } else {
        None
    };

    let left_side = container(import_btn)
        .width(Length::FillPortion(1))
        .center_x(Length::FillPortion(1));

    let center = container(rec_btn)
        .width(Length::Shrink)
        .center_x(Length::Shrink);

    let right_side = container(timer)
        .width(Length::FillPortion(1))
        .center_x(Length::FillPortion(1));

    let buttons_row = row![left_side, center, right_side]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let mut layout = column![input_row].spacing(8);
    if let Some(wave) = waveform {
        layout = layout.push(wave);
    }
    layout = layout.push(buttons_row);

    container(layout)
        .padding([12, 16])
        .width(Length::Fill)
        .style(theme::controls_container)
        .into()
}
