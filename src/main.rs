pub mod app;
pub mod config;

use iced::Theme;

use crate::{
    app::App,
    config::{Config, Set, Version},
};

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(Theme::Dark)
        .run()
    /*let config: Config = Config::new(
        Version::new(0, 1, 0),
        vec![Set::new(
            "Hot chicken wings".to_string(),
            "/home/bejmach/disc2/Images".to_string(),
            30,
            5,
            5,
        )],
    );
    let result = config.save(vec!["gdrawer"], "config");
    if let Err(content) = result {
        println!("ERROR: {}", content);
    }*/
}
