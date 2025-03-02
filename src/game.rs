use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, oneshot};

use log::{debug, error, info};

use crate::character::PlayerCharacter;
use crate::conn::Connection;
use crate::schema::{AIInput, QuestDefinition, QuestUpdate};

#[derive(Debug, Clone)]
pub enum GameError {
    ConnectionFailed,
    SendFailed(String),
    UnexpectedResponse(String),
    RefusalResponse(String),
    Custom(String),
}

#[derive(Debug)]
pub enum GamePlayer {
    GM,
    PC,
}

#[derive(Debug)]
enum GameMessage {
    Start {
        respond_to: oneshot::Sender<Result<(), GameError>>,
    },
    Input {
        respond_to: oneshot::Sender<Result<(), GameError>>,
        content: String,
    },
}

#[derive(Debug)]
pub struct GameLogEntry {
    pub player: GamePlayer,
    pub content: String,
}

impl GameLogEntry {
    pub fn new(player: GamePlayer, content: String) -> Self {
        Self { player, content }
    }
}

#[derive(Debug)]
pub struct GameBuilder {
    character: PlayerCharacter,
    api_key: Option<String>,
}

impl GameBuilder {
    pub fn new(character: PlayerCharacter) -> Self {
        Self {
            character,
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub async fn build(self) -> Result<GameHandle, GameError> {
        GameHandle::new(self).await
    }
}

#[derive(Debug, Clone)]
pub struct GameHandle {
    sender: mpsc::Sender<GameMessage>,
    state: Arc<RwLock<GameState>>,
}

impl GameHandle {
    async fn new(builder: GameBuilder) -> Result<Self, GameError> {
        let api_key = if let Some(key) = builder.api_key {
            key
        } else {
            std::env::var("OPENAI_API_KEY").unwrap()
        };
        let (sender, receiver) = mpsc::channel(8);
        let instance = GameInstance::new(receiver, api_key, builder.character).await?;
        let state = instance.state.clone();
        tokio::spawn(run_game(instance));
        Ok(Self { sender, state })
    }

    pub async fn start(&self) -> Result<(), GameError> {
        let (send, recv) = oneshot::channel();
        let msg = GameMessage::Start { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.unwrap()
    }

    pub async fn input(&self, content: String) -> Result<(), GameError> {
        let (send, recv) = oneshot::channel();
        let msg = GameMessage::Input {
            respond_to: send,
            content,
        };

        let _ = self.sender.send(msg).await;
        recv.await.unwrap()
    }

    pub fn state(&self) -> &Arc<RwLock<GameState>> {
        &self.state
    }
}

struct GameInstance {
    receiver: mpsc::Receiver<GameMessage>,
    connection: Connection,
    state: Arc<RwLock<GameState>>,
}

impl GameInstance {
    async fn new(
        receiver: mpsc::Receiver<GameMessage>,
        api_key: String,
        character: PlayerCharacter,
    ) -> Result<Self, GameError> {
        let connection = Connection::new(api_key).await;
        if let Err(error) = connection {
            error!("Connection failed: {}", error);
            return Err(GameError::ConnectionFailed);
        }

        info!("Connected!");

        Ok(Self {
            receiver,
            connection: connection.unwrap(),
            state: Arc::new(RwLock::new(GameState::new(character))),
        })
    }

    async fn process_update(&mut self, update: &QuestUpdate) {
        match update {
            QuestUpdate::QuestDefinition(def) => {
                let mut state = self.state.write().unwrap();
                (*state).quest = def.clone();
            }
            QuestUpdate::Description(desc) => {
                let mut state = self.state.write().unwrap();
                (*state)
                    .log
                    .push(GameLogEntry::new(GamePlayer::GM, desc.clone()));
            }
        }
    }

    async fn handle_message(&mut self, msg: GameMessage) {
        debug!("Handling message: {:?}", msg);
        match msg {
            GameMessage::Start { respond_to } => {
                let initial_message = {
                    let state = self.state.read().unwrap();
                    let pc = &(*state).character;
                    AIInput::Start(pc.clone())
                    // format!("I am a {} {} called {}, what is my quest?", pc.race(), pc.class(), pc.name())
                };
                let result = self.connection.send(initial_message).await;
                match result {
                    Ok(response) => {
                        for update in response.updates.iter() {
                            self.process_update(&update).await;
                        }
                        let _ = respond_to.send(Ok(()));
                    }
                    Err(error) => {
                        let _ = respond_to.send(Err(error));
                    }
                }
            }
            GameMessage::Input {
                respond_to,
                content,
            } => {
                {
                    let mut state = self.state.write().unwrap();
                    (*state)
                        .log
                        .push(GameLogEntry::new(GamePlayer::PC, content.clone()));
                    drop(state);
                }
                let result = self.connection.send(AIInput::UserInput(content)).await;
                match result {
                    Ok(response) => {
                        for update in response.updates.iter() {
                            self.process_update(&update).await;
                        }
                        let _ = respond_to.send(Ok(()));
                    }
                    Err(error) => {
                        let _ = respond_to.send(Err(error));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub character: PlayerCharacter,
    pub log: Vec<GameLogEntry>,
    pub quest: QuestDefinition,
}

impl GameState {
    fn new(character: PlayerCharacter) -> Self {
        Self {
            character,
            log: Vec::new(),
            quest: QuestDefinition::default(),
        }
    }
}

/// Async context that passes each `GameMessage` through to the `GameInstance`.
async fn run_game(mut instance: GameInstance) {
    while let Some(msg) = instance.receiver.recv().await {
        instance.handle_message(msg).await;
    }
}
