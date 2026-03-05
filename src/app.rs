use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use iced::widget::{
    Button, Column, Container, Row, Scrollable, Text, TextInput, button, center, column, container,
    image, row, text,
};
use iced::{Alignment, Element, Length, Task};
use rfd::{FileDialog, FileHandle};
use walkdir::WalkDir;

use crate::config::{Config, Set, Version};

#[derive(Debug)]
pub struct App {
    version: Version,

    screen: Screen,

    // Menu
    directory: String,
    images: Vec<PathBuf>,

    sets: BTreeMap<String, Set>,

    image_duration: u32,
    break_duration: u32,
    image_limit: u32,

    on_break: bool,

    timer: u32,

    display_time: DisplayTime,

    // Advanced
    set_name: String,
    set_directory: String,
    set_image_duration: u32,
    set_break_duration: u32,
    set_image_limit: u32,
}

#[derive(Debug)]
pub enum Screen {
    Menu,
    Advanced,
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

#[derive(Debug, Clone)]
pub enum AdvancedMessage {
    SetPressed(String),
    SetNameChanged(String),
    ImageDurationChanged(String),
    BreakDurationChanged(String),
    ImageLimitChanged(String),
    SaveSet,
    MenuPressed,
    StartPressed,
    ChangeDirectoryPressed,
    DirectoryChanged(Option<PathBuf>),
}

impl MenuMessage {
    pub fn to_mes(self) -> Message {
        Message::Menu(self)
    }
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
            sets: BTreeMap::new(),
            image_duration: 30,
            break_duration: 5,
            image_limit: 10,
            on_break: false,
            timer: 0,
            display_time: DisplayTime::Image,

            set_name: "Set".to_string(),
            set_directory: "".to_string(),
            set_image_duration: 30,
            set_break_duration: 5,
            set_image_limit: 10,
        }
    }
}

impl App {
    fn title(&self) -> String {
        "GDrawer".to_string()
    }

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
            MenuMessage::AdvancedPressed => {
                self.screen = Screen::Advanced;
                Task::none()
            }
            MenuMessage::StartPressed => Task::none(),
        }
    }
    fn advanced_update(&mut self, message: AdvancedMessage) -> Task<Message> {
        match message {
            AdvancedMessage::StartPressed => Task::none(),
            AdvancedMessage::MenuPressed => {
                self.screen = Screen::Menu;
                Task::none()
            }
            AdvancedMessage::SetNameChanged(name) => {
                self.set_name = name;
                Task::none()
            }
            AdvancedMessage::ImageDurationChanged(str_dur) => {
                if str_dur.is_empty() {
                    self.set_image_duration = 0;
                } else if let Ok(val) = str_dur.parse::<u32>() {
                    self.set_image_duration = val;
                }
                Task::none()
            }
            AdvancedMessage::BreakDurationChanged(str_dur) => {
                if str_dur.is_empty() {
                    self.set_break_duration = 0;
                } else if let Ok(val) = str_dur.parse::<u32>() {
                    self.set_break_duration = val;
                }
                Task::none()
            }
            AdvancedMessage::ImageLimitChanged(str_lim) => {
                if str_lim.is_empty() {
                    self.set_image_limit = 0;
                } else if let Ok(val) = str_lim.parse::<u32>() {
                    self.set_image_limit = val;
                }
                Task::none()
            }
            AdvancedMessage::SetPressed(set_name) => {
                if let Some(set) = self.sets.get(&set_name) {
                    self.set_image_duration = set.image_duration;
                    self.set_break_duration = set.break_duration;
                    self.set_image_limit = set.image_limit;
                }
                Task::none()
            }
            AdvancedMessage::SaveSet => {
                if self.sets.contains_key(&self.set_name) {
                    return Task::none();
                }
                let path = PathBuf::from_str(&self.set_directory);
                if path.is_err() {
                    return Task::none();
                }
                let dir_exist = fs::exists(path.unwrap());
                if self.set_name.is_empty() || dir_exist.is_err() || dir_exist.is_ok_and(|l| !l) {
                    return Task::none();
                }
                let set = Set::new(
                    self.set_name.clone(),
                    self.set_directory.clone(),
                    self.set_image_duration,
                    self.set_break_duration,
                    self.set_image_limit,
                );

                self.set_name = "Set".to_string();
                self.set_directory = "".to_string();
                self.set_image_duration = 30;
                self.set_break_duration = 5;
                self.set_image_limit = 10;
                println!("{:?}", set);
                self.sets.insert(set.name.clone(), set);
                println!("{:?}", self);
                Task::none()
            }
            AdvancedMessage::ChangeDirectoryPressed => {
                Task::perform(App::get_directory(), |path| {
                    AdvancedMessage::DirectoryChanged(path).to_mes()
                })
            }
            AdvancedMessage::DirectoryChanged(path) => {
                if let Some(path_buf) = path
                    && let Some(path_str) = path_buf.to_str()
                {
                    self.set_directory = path_str.to_string();
                    return Task::done(MenuMessage::ScanImagesPressed.to_mes());
                }
                Task::none()
            }
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
            Screen::Advanced => self.advanced_view(),
        }
    }

    fn menu_view(&self) -> Element<'_, Message> {
        let title: Text = Text::new(self.title())
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
    fn advanced_view(&self) -> Element<'_, Message> {
        let title: Text = Text::new(self.title())
            .size(40)
            .width(Length::Fill)
            .height(Length::Fixed(40.0))
            .center();

        // LEFT SIDE
        let set_list: Vec<Element<'_, Message>> = self
            .sets
            .values()
            .map(|v| {
                App::set_container(v)
                    .on_press(AdvancedMessage::SetPressed(v.name.clone()).to_mes())
                    .into()
            })
            .collect();
        println!("len: {}", set_list.len());
        let set_collumn = Column::with_children(set_list)
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(3);
        let set_scroll = Scrollable::new(set_collumn)
            .width(Length::Fill)
            .height(Length::Fill);
        let left_side = Container::new(set_scroll)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(container::bordered_box);

        // RIGHT SIDE

        let directory_button: Button<'_, Message> = Button::new(
            center("Select directory")
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .on_press(AdvancedMessage::ChangeDirectoryPressed.to_mes());

        let directory_text: Text = Text::new(format!("Directory: {}", self.set_directory))
            .width(Length::Fill)
            .height(Length::Shrink)
            .center();

        let dir_col = Column::with_capacity(3)
            .push(directory_button)
            .push(directory_text)
            .spacing(5)
            .width(Length::Fill)
            .height(Length::Shrink);

        let directory_container = Container::new(dir_col)
            .center_x(Length::Fill)
            .center_y(Length::Shrink)
            .style(container::bordered_box);

        let name_input = TextInput::new("", &self.set_name.to_string())
            .on_input(|content| AdvancedMessage::SetNameChanged(content).to_mes())
            .width(Length::Fill);
        let name_col = Column::new()
            .push(
                Container::new("Name:")
                    .align_left(Length::Fill)
                    .width(Length::Fixed(30.0)),
            )
            .push(name_input);
        let name_container: Container<'_, Message> = Container::new(name_col)
            .width(Length::Fill)
            .height(Length::Shrink);

        let image_input = TextInput::new("", &self.set_image_duration.to_string())
            .on_input(|content| AdvancedMessage::ImageDurationChanged(content).to_mes())
            .width(Length::Fill);
        let image_col = Column::new()
            .push(
                Container::new("Image time:")
                    .align_left(Length::Fill)
                    .height(Length::Fixed(30.0)),
            )
            .push(image_input);
        let image_container = Container::new(image_col)
            .width(Length::Fill)
            .height(Length::Shrink);

        let break_input = TextInput::new("", &self.set_break_duration.to_string())
            .on_input(|content| AdvancedMessage::BreakDurationChanged(content).to_mes())
            .width(Length::Fill);
        let break_col = Column::new()
            .push(
                Container::new("Break time:")
                    .align_left(Length::Fill)
                    .height(Length::Fixed(30.0)),
            )
            .push(break_input);
        let break_container = Container::new(break_col)
            .width(Length::Fill)
            .height(Length::Shrink);

        let limit_input = TextInput::new("", &self.set_image_limit.to_string())
            .on_input(|content| AdvancedMessage::ImageLimitChanged(content).to_mes())
            .width(Length::Fill);
        let limit_col = Column::new()
            .push(
                Container::new("Image limit:")
                    .align_left(Length::Fill)
                    .height(Length::Fixed(30.0)),
            )
            .push(limit_input);
        let limit_container = Container::new(limit_col)
            .width(Length::Fill)
            .height(Length::Shrink);

        let save_set: Button<'_, Message> =
            Button::new(center("Save Set").width(80.0).height(40.0))
                .on_press(AdvancedMessage::SaveSet.to_mes());

        let right_column = Column::new()
            .push(directory_container)
            .push(name_container)
            .push(image_container)
            .push(break_container)
            .push(limit_container)
            .push(save_set)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(5);

        let right_side = Container::new(right_column)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(container::bordered_box);

        let body_row = Row::new().push(left_side).push(right_side).spacing(5);

        let body = Container::new(body_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(15);

        let menu_button: Button<'_, Message> = Button::new(center("Go back").width(80).height(20))
            .on_press(AdvancedMessage::MenuPressed.to_mes());
        let start_button = Button::new(center("Start").width(80).height(20))
            .on_press(AdvancedMessage::StartPressed.to_mes());

        let bottom_row = Row::with_capacity(2)
            .push(menu_button)
            .push(start_button)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(5);

        Container::new(Column::new().push(title).push(body).push(bottom_row))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(15)
            .into()
    }
}
// Components
impl App {
    fn set_container(set: &Set) -> Button<'_, Message> {
        let name = Container::new(text(&set.name))
            .align_left(Length::FillPortion(2))
            .center_y(Length::Fill);
        let image_time =
            Container::new(text(format!("Image: {}", format_time(set.image_duration))))
                .center_x(Length::FillPortion(2))
                .center_y(Length::Fill);
        let break_time =
            Container::new(text(format!("Break: {}", format_time(set.break_duration))))
                .center_x(Length::FillPortion(2))
                .center_y(Length::Fill);
        let image_limit = Container::new(text(format!("Limit: {}", format_time(set.image_limit))))
            .center_x(Length::FillPortion(2))
            .center_y(Length::Fill);
        let directory = Container::new(text(format!("Directory: {}", set.directory)))
            .center_x(Length::FillPortion(6))
            .center_y(Length::Fill);
        let row = Row::new()
            .push(name)
            .push(image_time)
            .push(break_time)
            .push(image_limit)
            .push(directory)
            .width(Length::Fill)
            .height(Length::Fill);
        let component = Container::new(row)
            .width(Length::Fill)
            .height(Length::Fixed(40.0))
            .style(container::bordered_box);
        Button::new(component)
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
