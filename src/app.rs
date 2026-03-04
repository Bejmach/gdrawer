use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::widget::{
    Button, Column, Container, Row, Text, button, center, column, container, image, row, text,
};
use iced::{Alignment, Element, Length, Task};
use rfd::{FileDialog, FileHandle};
use walkdir::WalkDir;

use crate::config::{Config, Set, Version};

pub struct App {
    version: Version,

    screen: Screen,

    directory: String,
    images: Vec<PathBuf>,

    sets: HashMap<String, Set>,

    image_duration: u32,
    break_duration: u32,
    image_limit: u32,

    on_break: bool,

    timer: u32,

    display_time: DisplayTime,
}

pub enum Screen {
    Menu,
}

#[derive(Debug, Clone)]
pub enum DisplayTime {
    Image,
    Break,
}

#[derive(Debug, Clone)]
pub enum Message {
    Menu(MenuMessage),
    Advanced(AdvancedMessage),
}

#[derive(Debug, Clone)]
pub enum MenuMessage {
    ChangeDirectoryPressed,
    DirectoryChanged(Option<PathBuf>),
    ImageDurationChanged(u32),
    BreakDurationChanged(u32),
    ImageLimitChanged(u32),
    ScanImagesPressed,
    StartPressed,
    AdvancedPressed,
    ChangeDisplayedTime(DisplayTime),
}

impl MenuMessage {
    pub fn to_mes(self) -> Message {
        Message::Menu(self)
    }
}

#[derive(Debug, Clone)]
pub enum AdvancedMessage {
    SetPressed(String),
    LoadPressed,
}

impl AdvancedMessage {
    pub fn to_mes(self) -> Message {
        Message::Advanced(self)
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            version: Version::default(),
            screen: Screen::Menu,
            directory: "".to_string(),
            images: Vec::new(),
            sets: HashMap::new(),
            image_duration: 30,
            break_duration: 5,
            image_limit: 10,
            on_break: false,
            timer: 0,
            display_time: DisplayTime::Image,
        }
    }
}

impl App {
    fn get_config_path() -> Option<PathBuf> {
        let config_option: Option<PathBuf> = dirs_next::config_dir();
        if let Some(mut config_path) = config_option {
            config_path.push("gdrawer");
            config_path.push("config.json");
            return Some(config_path);
        }
        None
    }

    pub fn load_config(&mut self) {
        if let Some(config_path) = App::get_config_path() {
            if fs::exists(&config_path).is_err() {
                println!("Settings does not exists. Using default");
                return;
            }
            let content: String = fs::read_to_string(&config_path).unwrap();
            let config: Config = serde_json::from_str(&content).expect("settings are corrupted");
            self.sets = config.sets;
            self.version = config.version;
        } else {
            println!("Settings does not exists. Using default");
        }
    }
    pub fn save_config(&self) {
        if let Some(config_path) = App::get_config_path() {
            let content: String =
                serde_json::to_string(&Config::new(self.version, self.sets.clone()))
                    .expect("Cant parse config to json");
            fs::create_dir_all(&config_path).expect("Could not make directories");
            fs::write(&config_path, content).expect("Could not save config");
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Menu(menu_message) => self.menu_update(menu_message),
            Message::Advanced(advanced_message) => self.advanced_update(advanced_message),
        }
    }
    fn menu_update(&mut self, message: MenuMessage) -> Task<Message> {
        match message {
            MenuMessage::ChangeDirectoryPressed => Task::perform(App::get_directory(), |path| {
                MenuMessage::DirectoryChanged(path).to_mes()
            }),
            MenuMessage::DirectoryChanged(path) => {
                if let Some(path_buf) = path
                    && let Some(path_str) = path_buf.to_str()
                {
                    self.directory = path_str.to_string();
                    return Task::done(MenuMessage::ScanImagesPressed.to_mes());
                }
                Task::none()
            }
            MenuMessage::ScanImagesPressed => {
                if !self.directory.is_empty() {
                    self.images = get_images(&self.directory);
                }
                Task::none()
            }
            MenuMessage::ImageDurationChanged(duration) => {
                self.image_duration = duration;
                Task::none()
            }
            MenuMessage::BreakDurationChanged(duration) => {
                self.break_duration = duration;
                Task::none()
            }
            MenuMessage::ImageLimitChanged(limit) => {
                self.image_limit = limit;
                Task::none()
            }
            MenuMessage::ChangeDisplayedTime(display_time) => {
                self.display_time = display_time;
                Task::none()
            }
            _ => Task::none(),
        }
    }
    fn advanced_update(&mut self, message: AdvancedMessage) -> Task<Message> {
        match message {
            _ => Task::none(),
        }
    }
    async fn get_directory() -> Option<PathBuf> {
        let selected: Option<FileHandle> = rfd::AsyncFileDialog::new().pick_folder().await;
        if let Some(handler) = selected {
            return Some(handler.path().to_path_buf());
        }
        None
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Menu => self.menu_view(),
        }
    }
    fn menu_view(&self) -> Element<'_, Message> {
        let title: Text = Text::new("gDrawer")
            .size(40)
            .width(Length::Fill)
            .height(Length::Fixed(40.0))
            .center();

        let directory_button: Button<'_, Message> = Button::new(
            center("Select directory")
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .on_press(MenuMessage::ChangeDirectoryPressed.to_mes());

        let directory_text: Text = Text::new(format!("Directory: {}", self.directory))
            .width(Length::Fill)
            .height(Length::Shrink)
            .center();
        let found_images: Text = Text::new(format!("Found {} images", self.images.len()))
            .width(Length::Fill)
            .height(Length::Shrink)
            .center();

        let dir_col = Column::with_capacity(3)
            .push(directory_button)
            .push(directory_text)
            .push(found_images)
            .spacing(5)
            .width(Length::Fill)
            .height(Length::Shrink);

        let directory_container = Container::new(dir_col)
            .center_x(Length::Fill)
            .center_y(Length::Shrink)
            .style(container::bordered_box);

        let time_text: Text = match self.display_time {
            DisplayTime::Image => Text::new(format!(
                "Fixed time for all images: {}",
                format_time(self.image_duration)
            )),
            DisplayTime::Break => Text::new(format!(
                "Fixed time for all breaks: {}",
                format_time(self.break_duration)
            )),
        }
        .align_x(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Shrink);

        let time_buttons: Vec<Element<_>> = match self.display_time {
            DisplayTime::Image => {
                let image_times: Vec<u32> = vec![30, 60, 90, 120, 180];
                image_times
                    .iter()
                    .map(|&t| {
                        let label = format_time(t);
                        Button::new(center(text(label)).width(60).height(20))
                            .on_press(MenuMessage::ImageDurationChanged(t).to_mes())
                            .into()
                    })
                    .collect()
            }
            DisplayTime::Break => {
                let break_times: Vec<u32> = vec![5, 10, 15, 20, 30];
                break_times
                    .into_iter()
                    .map(|t| {
                        let label = format_time(t);
                        Button::new(center(text(label)).width(60).height(20))
                            .on_press(MenuMessage::BreakDurationChanged(t).to_mes())
                            .into()
                    })
                    .collect()
            }
        };

        let change_button: Button<'_, Message> = match self.display_time {
            DisplayTime::Image => Button::new(center("Break").width(60).height(20))
                .on_press(MenuMessage::ChangeDisplayedTime(DisplayTime::Break).to_mes()),
            DisplayTime::Break => Button::new(center("Images").width(60).height(20))
                .on_press(MenuMessage::ChangeDisplayedTime(DisplayTime::Image).to_mes()),
        };

        let button_row = Row::with_children(time_buttons)
            .push(change_button)
            .spacing(5)
            .width(Length::Shrink);

        let button_col = Column::with_capacity(2)
            .push(time_text)
            .push(button_row)
            .width(Length::Shrink)
            .align_x(Alignment::Start);

        let time_container = Container::new(button_col)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Start);

        let advanced_button = Button::new(center("Advanced").width(80).height(20))
            .on_press(MenuMessage::AdvancedPressed.to_mes());
        let start_button = Button::new(center("Start").width(80).height(20))
            .on_press(MenuMessage::StartPressed.to_mes());

        let bottom_row = Row::with_capacity(2)
            .push(advanced_button)
            .push(start_button)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(5);

        let bottom = Container::new(bottom_row)
            .align_right(Length::Fill)
            .height(Length::Shrink);

        let column = Column::with_capacity(4)
            .push(title)
            .push(directory_container)
            .push(time_container)
            .push(bottom)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5);

        Container::new(column)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(15)
            .into()
    }
}

fn format_time(time: u32) -> String {
    let seconds: u32 = time % 60;
    let minutes: u32 = time / 60;

    if minutes >= 60 {
        return "U retarded?".to_string();
    } else if time >= 100 {
        if seconds != 0 {
            return format!("{}min {}sec", minutes, seconds);
        } else {
            return format!("{}min", minutes);
        }
    }
    format!("{}sec", time)
}

fn get_images(path: &str) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| {
                    let ext = ext.to_string_lossy().to_lowercase();
                    ext == "png" || ext == "jpg" || ext == "jpeg"
                })
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}
