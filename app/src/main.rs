mod app;
mod audio;
mod clipboard;
mod history;
mod i18n;
mod models;
mod transcription;
mod translation;
mod tts;
mod types;
mod ui;

use iced::Size;

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Speech to Text")
        .subscription(app::App::subscription)
        .window_size(Size::new(420.0, 700.0))
        .theme(iced::Theme::Dark)
        .font(ui::icons::LUCIDE_BYTES)
        .run()
}
