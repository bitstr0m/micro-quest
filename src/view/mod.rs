use iced::task::Task;
use iced::theme::Theme;
use iced::Element;

mod character;
mod quest;

use character::CharacterCreate;
use quest::QuestLog;

pub fn main() -> iced::Result {
    iced::application("uQuest", update, view)
        .theme(|_| Theme::Dark)
        .exit_on_close_request(true)
        .run_with(State::new)
}

struct State {
    screen: Screen,
}

impl State {
    fn new() -> (Self, Task<Message>) {
        let (screen, task) = CharacterCreate::new();
        (
            Self {
                screen: Screen::CharacterCreate(screen),
            },
            task.map(Message::CharacterCreate),
        )
    }
}

enum Screen {
    CharacterCreate(character::CharacterCreate),
    Quest(quest::QuestLog),
}

#[derive(Debug, Clone)]
enum Message {
    CharacterCreate(character::Message),
    Quest(quest::Message),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::CharacterCreate(message) => {
            if let Screen::CharacterCreate(create) = &mut state.screen {
                if let Some(action) = create.update(message) {
                    match action {
                        character::Action::Run(task) => task.map(Message::CharacterCreate),
                        character::Action::Submit(pc) => {
                            let (quest, task) = QuestLog::new(pc);
                            state.screen = Screen::Quest(quest);
                            task.map(Message::Quest)
                        }
                    }
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        Message::Quest(message) => {
            if let Screen::Quest(quest) = &mut state.screen {
                if let Some(action) = quest.update(message) {
                    match action {
                        quest::Action::Run(task) => task.map(Message::Quest),
                    }
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
    }
}

fn view(state: &State) -> Element<Message> {
    match &state.screen {
        Screen::CharacterCreate(create) => create.view().map(Message::CharacterCreate),
        Screen::Quest(quest) => quest.view().map(Message::Quest),
    }
}
