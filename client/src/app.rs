use iced::alignment::{self, Alignment};
use iced::keyboard;
use iced::widget::{
    self, button, center, checkbox, column, container, keyed_column, row,
    scrollable, text, text_input, Text,
};
use iced::window;
use iced::{Command, Element, Font, Length, Subscription};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use realm_server::types::Message;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Default, Debug)]
pub enum RealmApp {
    #[default]
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
pub struct State {
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
        match self {
            RealmApp::Loading => {
                text_input::focus(INPUT_ID.clone())
            }
            RealmApp::Loaded(state) => {
                let command = match event {
                    Events::InputChanged(value) => {
                        state.input_value = value;

                        Command::none()
                    }
                    Events::SendMessage => {
                        //TODO

                        Command::none()
                    }
                };

                Command::batch(vec![command])
            }
        }
    }

    pub fn view(&self) -> Element<Events> {
        match self {
            RealmApp::Loading => loading_message(),
            RealmApp::Loaded(State {
                              input_value,
                              messages: tasks,
                              ..
                          }) => {
                let title = text("todos")
                    .width(Length::Fill)
                    .size(100)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(alignment::Horizontal::Center);

                let input = text_input("What needs to be done?", input_value)
                    .id(INPUT_ID.clone())
                    .on_input(Events::InputChanged)
                    .on_submit(Events::SendMessage)
                    .padding(15)
                    .size(30);

                // let controls = view_controls(tasks, *filter);
                // let filtered_tasks =
                //     tasks.iter().filter(|task| filter.matches(task));
                // 
                // let tasks: Element<_> = if filtered_tasks.count() > 0 {
                //     keyed_column(
                //         tasks
                //             .iter()
                //             .enumerate()
                //             .filter(|(_, task)| filter.matches(task))
                //             .map(|(i, task)| {
                //                 (
                //                     task.id,
                //                     task.view(i).map(move |message| {
                //                         Events::TaskMessage(i, message)
                //                     }),
                //                 )
                //             }),
                //     )
                //         .spacing(10)
                //         .into()
                // } else {
                //     empty_message(match filter {
                //         Filter::All => "You have not created a task yet...",
                //         Filter::Active => "All your tasks are done! :D",
                //         Filter::Completed => {
                //             "You have not completed a task yet..."
                //         }
                //     })
                // };

                let content = column![title, input]
                    .spacing(20)
                    .max_width(800);

                scrollable(
                    container(content).center_x(Length::Fill).padding(40),
                )
                    .into()
            }
        }
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