use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use iced::widget::image::Handle;
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Text, TextInput, button, center, center_x,
    column, container, row, text,
};
use iced::{Alignment, Element, Length, Subscription, Task, Transformation, time};
use rand::Rng;
use rand::seq::SliceRandom;
use rfd::{FileDialog, FileHandle};
use walkdir::WalkDir;

use image::ImageReader;

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

    running: bool,
    on_break: bool,

    cur_handle: Option<Handle>,
    next_handle: Option<Handle>,
    cur_id: Option<u32>,
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
    Image,
    Break,
    Finish,
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
    Image(ImageMessage),
    Break(BreakMessage),
    Finish(FinishMessage),
    Tick,
    LoadImage,
    CurImageLoaded(Handle),
    PreloadImage,
    NextImageLoaded(Handle),
    LoadBothImages,
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
    DeleteSet(String),
    MenuPressed,
    StartPressed,
    ChangeDirectoryPressed,
    DirectoryChanged(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub enum ImageMessage {
    SkipImage,
    BreakSession,
}

#[derive(Debug, Clone)]
pub enum BreakMessage {
    EndBreak,
    BreakSession,
}

#[derive(Debug, Clone)]
pub enum FinishMessage {
    MenuPressed,
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

impl ImageMessage {
    pub fn to_mes(self) -> Message {
        Message::Image(self)
    }
}

impl BreakMessage {
    pub fn to_mes(self) -> Message {
        Message::Break(self)
    }
}
impl FinishMessage {
    pub fn to_mes(self) -> Message {
        Message::Finish(self)
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
            running: false,
            on_break: false,
            cur_handle: None,
            next_handle: None,
            cur_id: None,
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

// Base
impl App {
    pub fn new() -> Self {
        let config_result = Config::load(vec!["gdrawer"], "data");
        let (version, sets) = if let Ok(config) = config_result {
            (config.version, config.sets.clone())
        } else {
            (Version::default(), BTreeMap::new())
        };

        Self {
            version,
            screen: Screen::Menu,
            directory: "".to_string(),
            images: Vec::new(),
            sets,
            image_duration: 30,
            break_duration: 5,
            image_limit: 10,
            running: false,
            on_break: false,
            cur_handle: None,
            next_handle: None,
            cur_id: None,
            timer: 0,
            display_time: DisplayTime::Image,

            set_name: "Set".to_string(),
            set_directory: "".to_string(),
            set_image_duration: 30,
            set_break_duration: 5,
            set_image_limit: 10,
        }
    }

    pub fn title(&self) -> String {
        "GDrawer".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Menu(menu_message) => self.menu_update(menu_message),
            Message::Advanced(advanced_message) => self.advanced_update(advanced_message),
            Message::Image(image_message) => self.image_update(image_message),
            Message::Break(break_message) => self.break_update(break_message),
            Message::Finish(finish_message) => self.finish_update(finish_message),
            Message::Tick => self.on_tick(),
            Message::LoadImage => Task::perform(
                App::get_handle(
                    self.images
                        .get((self.cur_id.unwrap_or_default()) as usize)
                        .unwrap()
                        .clone(),
                ),
                Message::CurImageLoaded,
            ),
            Message::CurImageLoaded(handle) => {
                self.cur_handle = Some(handle);
                println!("{:?}", self.cur_handle);
                Task::none()
            }
            Message::PreloadImage => Task::perform(
                App::get_handle(
                    self.images
                        .get((self.cur_id.unwrap_or_default() + 1) as usize)
                        .unwrap()
                        .clone(),
                ),
                Message::NextImageLoaded,
            ),
            Message::NextImageLoaded(handle) => {
                self.next_handle = Some(handle);
                println!("{:?}", self.next_handle);
                Task::none()
            }
            Message::LoadBothImages => {
                let cur = self.cur_id.unwrap_or(0) as usize;

                let cur_path = self.images[cur].clone();
                let next_path = self.images[cur + 1].clone();

                println!("Load both");

                Task::batch(vec![
                    Task::perform(App::get_handle(cur_path), Message::CurImageLoaded),
                    Task::perform(App::get_handle(next_path), Message::NextImageLoaded),
                ])
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Menu => self.menu_view(),
            Screen::Advanced => self.advanced_view(),
            Screen::Image => self.image_view(),
            Screen::Break => self.break_view(),
            Screen::Finish => self.finish_view(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.running {
            time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

//Functions
impl App {
    async fn get_directory() -> Option<PathBuf> {
        let selected: Option<FileHandle> = rfd::AsyncFileDialog::new().pick_folder().await;
        if let Some(handler) = selected {
            return Some(handler.path().to_path_buf());
        }
        None
    }
    async fn get_handle(path: PathBuf) -> Handle {
        let img = ImageReader::open(&path)
            .expect("open failed")
            .decode()
            .expect("decode failed")
            .to_rgba8();

        println!("decoded: {:?}", path);

        let (width, height) = img.dimensions();

        Handle::from_rgba(width, height, img.into_raw())
    }
    fn save_data(&self) -> Result<()> {
        let config: Config = Config::new(self.version, self.sets.clone());
        config.save(vec!["gdrawer"], "data")
    }
    fn shuffle_img(&mut self) {
        self.images.shuffle(&mut rand::rng());
    }
    fn on_tick(&mut self) -> Task<Message> {
        self.timer -= 1;
        if self.timer == 0 {
            if self.on_break {
                self.move_handle();
                self.timer = self.image_duration;
                self.screen = Screen::Image;
                self.on_break = false;
                return Task::done(Message::PreloadImage);
            } else {
                let id: u32 = self.cur_id.unwrap_or_default() + 1;
                if id >= self.image_limit {
                    self.finish_session();
                    return Task::none();
                } else {
                    self.enable_break();
                    return Task::none();
                }
            }
        }
        Task::none()
    }
    fn enable_break(&mut self) {
        self.cur_id = Some(self.cur_id.unwrap_or_default() + 1);
        self.on_break = true;
        self.screen = Screen::Break;
        self.timer = self.break_duration;
    }
    fn finish_session(&mut self) {
        self.running = false;
        self.screen = Screen::Finish;
    }

    fn move_handle(&mut self) {
        self.cur_handle = self.next_handle.take();
        self.next_handle = None;
    }
    fn start(&mut self) {
        self.image_limit = self.image_limit.min(self.images.len() as u32);
        self.cur_id = Some(0);
        self.running = true;
        self.shuffle_img();
        self.screen = Screen::Image;
        self.timer = self.image_duration;
    }
}

//Menu
impl App {
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
            MenuMessage::StartPressed => {
                self.start();
                Task::done(Message::LoadBothImages)
            }
        }
    }
}

//Advanced
impl App {
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
                App::set_container(
                    v,
                    AdvancedMessage::SetPressed(v.name.clone()).to_mes(),
                    AdvancedMessage::DeleteSet(v.name.clone()).to_mes(),
                )
            })
            .collect();
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
        let footer = Container::new(bottom_row)
            .align_right(Length::Fill)
            .height(Length::Shrink);

        Container::new(Column::new().push(title).push(body).push(footer))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(15)
            .into()
    }
    fn advanced_update(&mut self, message: AdvancedMessage) -> Task<Message> {
        match message {
            AdvancedMessage::StartPressed => {
                self.image_duration = self.set_image_duration;
                self.break_duration = self.set_break_duration;
                self.image_limit = self.set_image_limit;
                self.directory = self.set_directory.clone();
                self.images = get_images(&self.directory);
                self.start();
                Task::done(Message::LoadBothImages)
            }
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
                    self.set_name = set.name.clone();
                    self.set_directory = set.directory.clone();
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
                self.sets.insert(set.name.clone(), set);
                let _ = self.save_data();
                Task::none()
            }
            AdvancedMessage::DeleteSet(set_name) => {
                self.sets.remove(&set_name);
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
}

//Image
impl App {
    fn image_view(&self) -> Element<'_, Message> {
        let image: Container<'_, Message> = if let Some(handle) = &self.cur_handle {
            Container::new(Image::new(handle).expand(true))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        } else {
            center("Failed to get image")
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let timer = center(text(format_time(self.timer)))
            .width(Length::Fill)
            .height(Length::Fixed(40.0));

        let skip_button = Button::new("Skip")
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(30.0))
            .on_press(ImageMessage::SkipImage.to_mes());
        let break_button = Button::new("Break")
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(30.0))
            .on_press(ImageMessage::BreakSession.to_mes());

        let spacer = Container::new("").width(Length::Fill);

        let footer_row = Row::new()
            .push(break_button)
            .push(spacer)
            .push(skip_button)
            .spacing(5);
        let footer: Container<'_, Message> = Container::new(footer_row)
            .center_x(Length::Fill)
            .height(Length::Fixed(30.0));

        let view_column = Column::new().push(image).push(timer).push(footer);
        Container::new(view_column)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
    fn image_update(&mut self, message: ImageMessage) -> Task<Message> {
        match message {
            ImageMessage::SkipImage => {
                let id = self.cur_id.unwrap_or_default() + 1;
                if id >= self.image_limit {
                    self.finish_session();
                } else if self.next_handle.is_some() {
                    self.move_handle();
                    self.timer = self.image_duration + 1;
                    self.cur_id = Some(id);
                    if id < self.image_limit - 1 {
                        return Task::done(Message::PreloadImage);
                    } else {
                        return Task::none();
                    }
                } else {
                    return Task::none();
                }

                Task::none()
            }
            ImageMessage::BreakSession => {
                self.running = false;
                self.screen = Screen::Menu;
                self.cur_id = None;
                Task::none()
            }
        }
    }
}

//Break
impl App {
    fn break_view(&self) -> Element<'_, Message> {
        let break_text: Container<'_, Message> = center(Text::new("Break Time").size(40));

        let timer = center(text(format_time(self.timer)))
            .width(Length::Fill)
            .height(Length::Fixed(40.0));

        let skip_button = Button::new("Skip")
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(30.0))
            .on_press(BreakMessage::EndBreak.to_mes());
        let break_button = Button::new("Break")
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(30.0))
            .on_press(BreakMessage::BreakSession.to_mes());

        let spacer = Container::new("").width(Length::Fill);

        let footer_row = Row::new()
            .push(break_button)
            .push(spacer)
            .push(skip_button)
            .spacing(5);
        let footer: Container<'_, Message> = Container::new(footer_row)
            .center_x(Length::Fill)
            .align_bottom(Length::Shrink);

        let view_column = Column::new()
            .push(Container::new("").height(Length::Fill))
            .push(break_text)
            .push(timer)
            .push(Container::new("").height(Length::Fill))
            .push(footer);
        Container::new(view_column)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(15)
            .into()
    }
    fn break_update(&mut self, message: BreakMessage) -> Task<Message> {
        match message {
            BreakMessage::EndBreak => {
                self.move_handle();
                self.on_break = false;
                self.screen = Screen::Image;
                self.timer = self.image_duration + 1;
                Task::done(Message::PreloadImage)
            }
            BreakMessage::BreakSession => {
                self.running = false;
                self.screen = Screen::Menu;
                self.cur_id = None;
                Task::none()
            }
        }
    }
}

//Finish
impl App {
    fn finish_view(&self) -> Element<'_, Message> {
        let finish_text: Container<'_, Message> =
            center(Text::new("Session ended. Take some break").size(40))
                .width(Length::Fill)
                .height(Length::Fill);

        let back_button = Button::new("Back to menu").on_press(FinishMessage::MenuPressed.to_mes());

        let footer_row = Row::new().push(back_button).spacing(5);
        let footer = Container::new(footer_row)
            .center_x(Length::Fill)
            .align_bottom(Length::Shrink);

        let view_column = Column::new().push(finish_text).push(footer);
        Container::new(view_column)
            .center(Length::Fill)
            .padding(15.0)
            .into()
    }
    fn finish_update(&mut self, message: FinishMessage) -> Task<Message> {
        match message {
            FinishMessage::MenuPressed => {
                self.screen = Screen::Menu;
                self.running = false;
                self.on_break = false;
                Task::none()
            }
        }
    }
}

// Components
impl App {
    fn set_container(
        set: &Set,
        set_message: Message,
        _delete_message: Message,
    ) -> Element<'_, Message> {
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
        Button::new(component).on_press(set_message).into()
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
