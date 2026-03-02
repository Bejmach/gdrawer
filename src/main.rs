mod app;

use iced::Theme;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(Theme::Dark)
        .run()
}
