use iced::alignment::Horizontal;
use iced::task::Task;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, text_input, vertical_space, Column,
};
use iced::{color, Center, Element, Fill};
use iced_aw::widgets::spinner::Spinner;

use crate::character::PlayerCharacter;
use crate::game::{GameBuilder, GameError, GameHandle, GameLogEntry, GamePlayer};
use crate::schema::QuestDefinition;

#[derive(Debug, Clone)]
pub(super) enum Message {
    Loaded(Result<GameHandle, GameError>),
    Started(Result<(), GameError>),
    InputFieldChange(String),
    InputSubmit,
    Response,
}

pub(super) enum Action {
    Run(Task<Message>),
}

#[derive(Debug, Default)]
pub(super) struct QuestLog {
    game: Option<GameHandle>,
    input_field: String,
    waiting: bool,
}

impl QuestLog {
    pub(super) fn new(character: PlayerCharacter) -> (Self, Task<Message>) {
        let game_builder = GameBuilder::new(character);
        (
            Self {
                game: None,
                input_field: String::new(),
                waiting: true,
            },
            Task::perform(game_builder.build(), Message::Loaded),
        )
    }

    pub(super) fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            Message::Loaded(game) => {
                let game = game.expect("Error loading quest");
                self.game = Some(game.clone());
                Some(Action::Run(Task::perform(
                    async move { game.start().await },
                    Message::Started,
                )))
            }
            Message::Started(_) => {
                self.waiting = false;
                None
            }
            Message::InputFieldChange(content) => {
                self.input_field = content;
                None
            }
            Message::InputSubmit => {
                if let Some(game) = &self.game {
                    let game = game.clone();
                    let content = self.input_field.clone();
                    self.input_field = String::new();
                    self.waiting = true;
                    Some(Action::Run(
                        Task::perform(async move { game.input(content).await }, |_| {
                            Message::Response
                        })
                        .chain(scrollable::snap_to(
                            scrollable::Id::new("game-log"),
                            scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                        )),
                    ))
                } else {
                    None
                }
            }
            Message::Response => {
                self.waiting = false;
                Some(Action::Run(scrollable::snap_to(
                    scrollable::Id::new("game-log"),
                    scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                )))
            }
        }
    }

    pub(super) fn view(&self) -> Element<Message> {
        if let Some(game) = &self.game {
            let state = game.state().read().unwrap();
            column![
                scrollable(column![
                    self.view_quest_summary(&state.quest),
                    vertical_space().height(10),
                    Column::with_children(
                        state
                            .log
                            .iter()
                            .map(|entry| column![
                                self.view_log_entry(&entry),
                                vertical_space().height(20)
                            ])
                            .map(Element::from)
                    )
                ])
                .width(Fill)
                .height(Fill)
                .spacing(20)
                .id(scrollable::Id::new("game-log")),
                vertical_space().height(20),
                if self.waiting {
                    Element::from(Spinner::default())
                } else {
                    Element::from(
                        text_input("What would you like to do?", &self.input_field)
                            .width(Fill)
                            .on_input(Message::InputFieldChange)
                            .on_submit(Message::InputSubmit),
                    )
                },
            ]
            .width(Fill)
            .align_x(Center)
            .padding(20)
            .into()
        } else {
            text("Loading...").into()
        }
    }

    fn view_log_entry(&self, entry: &GameLogEntry) -> Element<Message> {
        let player_text = match entry.player {
            GamePlayer::GM => "GM:",
            GamePlayer::PC => "PC:",
        };
        row![
            text(player_text)
                .color(color!(0x666666))
                .align_x(Horizontal::Left)
                .width(60),
            text(entry.content.clone()).width(Fill),
        ]
        .into()
    }

    fn view_quest_summary(&self, quest: &QuestDefinition) -> Element<Message> {
        row![
            horizontal_space().width(60),
            container(column![
                text(quest.title.clone())
                    .size(24)
                    .align_x(Horizontal::Center),
                vertical_space().height(5),
                text(quest.description.clone()).align_x(Horizontal::Left),
                vertical_space().height(5),
                text(quest.objective_summary.clone()).align_x(Horizontal::Left),
            ],)
            .padding(10)
            .style(container::bordered_box),
        ]
        .into()
    }
}
