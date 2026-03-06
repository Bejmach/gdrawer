pub mod app;
pub mod config;

use iced::Theme;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .theme(Theme::Dark)
        .run()
}
