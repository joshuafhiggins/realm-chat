use iced::alignment::{self, Alignment};
use iced::keyboard;
use iced::widget::{
    self, button, center, checkbox, column, container, keyed_column, row,
    scrollable, text, text_input, Text,
};
use iced::window;
use iced::{Command, Element, Font, Length, Subscription};
use iced::futures::StreamExt;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use realm_server::types::Message;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Default, Debug)]
pub struct RealmApp {
    input_value: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub enum Events {
    // Loaded(Result<SavedState, LoadError>),
    // Saved(Result<(), SaveError>),
    InputChanged(String),
    SendMessage,
}

impl RealmApp {
    pub fn load() -> Command<Events> {
        //Command::perform(SavedState::load(), Events::Loaded)
        Command::none()
    }

    pub fn title(&self) -> String {
        "Realm Chat".to_string()
    }

    pub fn update(&mut self, event: Events) -> Command<Events> {
        let command = match event {
            Events::InputChanged(value) => {
                self.input_value = value;

                Command::none()
            }
            Events::SendMessage => {
                //TODO
                self.input_value.clear();
                Command::none()
            }
        };

        Command::batch(vec![command])
    }

    pub fn view(&self) -> Element<Events> {
        let title = text("todos")
            .width(Length::Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(alignment::Horizontal::Center);

        let input = text_input("New Message", &*self.input_value)
            .id(INPUT_ID.clone())
            .on_input(Events::InputChanged)
            .on_submit(Events::SendMessage)
            //.padding(15)
            .size(12);

        let messages: Vec<Text> = Vec::new();
        for message in &self.messages {
            
        }
        
        let content = column![title, input]
            .spacing(20);
            //.max_width(800);

        scrollable(
            container(content).center_x(Length::Fill).padding(40),
        ).into()
    }

    pub fn subscription(&self) -> Subscription<Events> {
        use keyboard::key;

        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                // (key::Named::Tab, _) => Some(Events::TabPressed {
                //     shift: modifiers.shift(),
                // }),
                // (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                //     Some(Events::ToggleFullscreen(window::Mode::Fullscreen))
                // }
                // (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                //     Some(Events::ToggleFullscreen(window::Mode::Windowed))
                // }
                _ => None,
            }
        })
    }
}

fn loading_message<'a>() -> Element<'a, Events> {
    center(
        text("Loading...")
            .horizontal_alignment(alignment::Horizontal::Center)
            .size(50),
    )
        .into()
}

fn empty_message(message: &str) -> Element<'_, Events> {
    center(
        text(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .color([0.7, 0.7, 0.7]),
    )
        .height(200)
        .into()
}

// Fonts
const ICONS: Font = Font::with_name("Iced-Todos-Icons");

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}

fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}