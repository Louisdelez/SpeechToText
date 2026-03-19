use iced::widget::{column, container, text, Space};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::i18n::Strings;
use crate::types::{AppMode, ModelState};
use crate::ui::theme;

pub fn view<'a>(model_state: &ModelState, mode: AppMode, s: &'a Strings) -> Element<'a, Message> {
    let (status, hint) = match model_state {
        ModelState::NotDownloaded => (s.preparing_model.to_string(), "".to_string()),
        ModelState::Downloading { progress_pct } => (
            format!("{} {}%", s.downloading_model, progress_pct),
            "".to_string(),
        ),
        ModelState::Loading => (s.loading_model.to_string(), "".to_string()),
        ModelState::Ready => {
            match mode {
                AppMode::SpeechToText => (
                    s.ready.to_string(),
                    s.ready_hint.to_string(),
                ),
                AppMode::Translation => (
                    "Pret a traduire".to_string(),
                    "Parlez ou ecrivez un texte a traduire".to_string(),
                ),
                AppMode::Summary => (
                    "Pret a resumer".to_string(),
                    "Parlez ou collez un texte a resumer".to_string(),
                ),
                AppMode::Corrector => (
                    "Correcteur".to_string(),
                    "Ecrivez ou collez un texte a corriger".to_string(),
                ),
                AppMode::PromptEngineer => (
                    "Prompt Engineer".to_string(),
                    "Decrivez ce que vous voulez, je cree le prompt optimise".to_string(),
                ),
                AppMode::TextToSpeech => (
                    "Text to Speech".to_string(),
                    "Ecrivez un texte a convertir en audio".to_string(),
                ),
            }
        }
        ModelState::Error(e) => (
            format!("{}{e}", s.error_prefix),
            s.check_internet.to_string(),
        ),
    };

    let content = column![
        Space::new().height(Length::Fill),
        text(status).size(15).style(theme::secondary_text),
        text(hint).size(13).style(theme::muted_text),
        Space::new().height(Length::Fill),
    ]
    .spacing(8)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
