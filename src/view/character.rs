use iced::alignment::{Horizontal, Vertical};
use iced::color;
use iced::task::Task;
use iced::widget::{
    button, column, markdown, row, scrollable, text, text_input, vertical_space, Column, Row,
};
use iced::Center;
use iced::Element;
use iced::Fill;

use crate::character::{PlayerCharacter, PlayerCharacterBuilder};

#[derive(Debug, Clone)]
pub(super) enum Message {
    NameChange(String),
    RaceChange(String),
    ClassChange(String),
    Submit,
}

pub(super) enum Action {
    Run(Task<Message>),
    Submit(PlayerCharacter),
}

#[derive(Debug, Default)]
pub(super) struct CharacterCreate {
    name_field: String,
    race_field: String,
    class_field: String,
}

impl CharacterCreate {
    pub(super) fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub(super) fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            Message::NameChange(content) => {
                self.name_field = content;
                None
            }
            Message::RaceChange(content) => {
                self.race_field = content;
                None
            }
            Message::ClassChange(content) => {
                self.class_field = content;
                None
            }
            Message::Submit => {
                let name = if self.name_field.trim().is_empty() {
                    "Jim".to_owned()
                } else {
                    self.name_field.trim().to_owned()
                };
                let race = if self.race_field.trim().is_empty() {
                    "Human".to_owned()
                } else {
                    self.race_field.trim().to_owned()
                };
                let class = if self.class_field.trim().is_empty() {
                    "Fighter".to_owned()
                } else {
                    self.class_field.trim().to_owned()
                };
                let pc = PlayerCharacterBuilder::new(name)
                    .with_race(race)
                    .with_class(class)
                    .build();
                Some(Action::Submit(pc))
            }
        }
    }

    pub(super) fn view(&self) -> Element<Message> {
        column![
            vertical_space().height(100),
            row![
                text("Name:").align_x(Horizontal::Left).width(60),
                text_input("Jim", &self.name_field).on_input(Message::NameChange),
            ]
            .align_y(Vertical::Center),
            vertical_space().height(40),
            row![
                text("Race:").align_x(Horizontal::Left).width(60),
                text_input("Human", &self.race_field).on_input(Message::RaceChange),
            ]
            .align_y(Vertical::Center),
            vertical_space().height(40),
            row![
                text("Class:").align_x(Horizontal::Left).width(60),
                text_input("Fighter", &self.class_field).on_input(Message::ClassChange),
            ]
            .align_y(Vertical::Center),
            vertical_space().height(40),
            button("Submit").width(100).on_press(Message::Submit),
        ]
        .width(Fill)
        .align_x(Center)
        .padding(20)
        .into()
    }
}
