use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::widget::{Container, button, column, image, row, text};
use iced::{Element, Length, Task};
use rfd::{FileDialog, FileHandle};
use walkdir::WalkDir;

use crate::config::{Config, Set, Version};

pub struct App {
    version: Version,

    screen: Screen,

    directory: String,
    images: Vec<PathBuf>,

    sets: Vec<Set>,

    image_duration: u32,
    break_duration: u32,
    image_limit: u32,

    on_break: bool,

    timer: u32,
}

pub enum Screen {
    Menu,
}
#[derive(Debug, Clone)]
pub enum Message {
    Menu(MenuMessage),
}

#[derive(Debug, Clone)]
pub enum MenuMessage {
    ChangeDirectoryPressed,
    DirectoryChanged(Option<PathBuf>),
    ScanImagesPressed,
    ImageDurationChanged(u32),
    BreakDurationChanged(u32),
    ImageLimitChanged(u32),
    StartPressed,
}

impl MenuMessage {
    fn to_mes(self) -> Message {
        Message::Menu(self)
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            version: Version::default(),
            screen: Screen::Menu,
            directory: "".to_string(),
            images: Vec::new(),
            sets: Vec::new(),
            image_duration: 30,
            break_duration: 5,
            image_limit: 10,
            on_break: false,
            timer: 0,
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
                serde_json::to_string(&Config::new(self.version, self.sets.to_vec()))
                    .expect("Cant parse config to json");
            fs::create_dir_all(&config_path).expect("Could not make directories");
            fs::write(&config_path, content).expect("Could not save config");
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Menu(menu_message) => self.menu_update(menu_message),
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
        Container::new(column![
            Container::new(column![
                text(format!("Current directory: {}", self.directory)),
                text(format!("Found {} images", self.images.len())),
                row![
                    button("Change directory")
                        .on_press(MenuMessage::ChangeDirectoryPressed.to_mes()),
                    button("Scan for images").on_press(MenuMessage::ScanImagesPressed.to_mes()),
                ]
                .spacing(5),
            ])
            .center_x(Length::Fill)
            .height(Length::FillPortion(2)),
            Container::new(column![
                Container::new(column![
                    text(format!("Image time: {}", self.image_duration)),
                    row![
                        button("30sec").on_press(MenuMessage::ImageDurationChanged(30).to_mes()),
                        button("60sec").on_press(MenuMessage::ImageDurationChanged(60).to_mes()),
                        button("90sec").on_press(MenuMessage::ImageDurationChanged(90).to_mes()),
                        button("2min").on_press(MenuMessage::ImageDurationChanged(120).to_mes()),
                        button("3min").on_press(MenuMessage::ImageDurationChanged(180).to_mes()),
                    ]
                    .spacing(5)
                ])
                .center_x(Length::Fill),
                Container::new(column![
                    text(format!("Break time: {}", self.break_duration)),
                    row![
                        button("5sec").on_press(MenuMessage::BreakDurationChanged(5).to_mes()),
                        button("10sec").on_press(MenuMessage::BreakDurationChanged(10).to_mes()),
                        button("15sec").on_press(MenuMessage::BreakDurationChanged(15).to_mes()),
                        button("20sec").on_press(MenuMessage::BreakDurationChanged(20).to_mes()),
                    ]
                    .spacing(5)
                ])
                .center_x(Length::Fill)
            ])
            .center_x(Length::Fill)
            .height(Length::FillPortion(3))
        ])
        .center_x(Length::Fill)
        .height(Length::Fill)
        .into()
    }
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
