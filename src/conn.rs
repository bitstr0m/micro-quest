use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantsApiResponseFormatOption, CreateAssistantRequestArgs, CreateMessageRequestArgs,
        CreateRunRequestArgs, CreateThreadRequestArgs, MessageContent, MessageRole,
        ModifyAssistantRequestArgs, ResponseFormat, ResponseFormatJsonSchema, RunStatus,
    },
    Client,
};

use log::{debug, error, info};

use schemars::{schema_for, JsonSchema};

use crate::game::GameError;
use crate::schema::{AIInput, AIOutput};

pub struct Connection {
    client: Client<OpenAIConfig>,
    assistant_id: String,
    thread_id: String,
}

impl Connection {
    pub async fn new(_api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let assistant_id = Self::get_assistant(&client).await?;
        let thread_request = CreateThreadRequestArgs::default().build()?;
        let thread = client.threads().create(thread_request.clone()).await?;
        let thread_id = thread.id.clone();
        Ok(Self {
            client,
            assistant_id,
            thread_id,
        })
    }

    async fn get_assistant(
        client: &Client<OpenAIConfig>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let list_assistant_query: [usize; 0] = [];
        let assistants = client.assistants();
        let assistant_list = assistants.list(&list_assistant_query).await?;
        let instructions = Self::get_assistant_instructions();
        if let Some(assistant) = assistant_list
            .data
            .iter()
            .find(|a| a.name == Some(AI_NAME.to_owned()))
        {
            info!("Found OpenAI assistant");
            if assistant.instructions != Some(instructions.clone()) {
                info!("Assistant instruction mismatch, updating assistant");
                assistants
                    .update(
                        &assistant.id,
                        ModifyAssistantRequestArgs::default()
                            .name(AI_NAME)
                            .model(AI_MODEL)
                            .instructions(instructions)
                            .response_format(AssistantsApiResponseFormatOption::Format(
                                Self::get_assistant_response_format(),
                            ))
                            .build()?,
                    )
                    .await?;
            }
            Ok(assistant.id.clone())
        } else {
            info!("Creating new OpenAI assistant");
            let assistant = assistants
                .create(
                    CreateAssistantRequestArgs::default()
                        .name(AI_NAME)
                        .model(AI_MODEL)
                        .instructions(instructions)
                        .response_format(AssistantsApiResponseFormatOption::Format(
                            Self::get_assistant_response_format(),
                        ))
                        .build()?,
                )
                .await?;
            Ok(assistant.id.clone())
        }
    }

    fn get_assistant_instructions() -> String {
        let schema = schema_for!(AIInput);
        let schema_value = serde_json::to_string(&schema).unwrap();
        let mut inst = AI_INST.to_owned();
        inst.push_str(&schema_value);
        inst.push_str(AI_INST_PROLOGUE);
        return inst;
    }

    fn get_assistant_response_format() -> ResponseFormat {
        let schema = schema_for!(AIOutput);
        let schema_value = serde_json::to_value(&schema).unwrap();
        debug!(
            "Schema:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: Some(AI_RESPONSE_DESC.to_owned()),
                name: "quest".to_owned(),
                schema: Some(schema_value),
                strict: Some(false),
            },
        }
    }

    pub async fn send(&self, command: AIInput) -> Result<AIOutput, GameError> {
        debug!("Sending: {:?}", &command);
        let command_str = serde_json::to_string(&command).unwrap();
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(command_str)
            .build()
            .map_err(|_| GameError::SendFailed("Could not build message".to_owned()))?;

        let _ = self
            .client
            .threads()
            .messages(&self.thread_id)
            .create(message)
            .await
            .map_err(|_| GameError::SendFailed("Could not add message to thread".to_owned()))?;
        let run = self
            .client
            .threads()
            .runs(&self.thread_id)
            .create(
                CreateRunRequestArgs::default()
                    .assistant_id(&self.assistant_id)
                    .build()
                    .map_err(|_| GameError::SendFailed("Could not build run request".to_owned()))?,
            )
            .await
            .map_err(|_| GameError::SendFailed("Could not create run".to_owned()))?;

        loop {
            let run = self
                .client
                .threads()
                .runs(&self.thread_id)
                .retrieve(&run.id)
                .await
                .map_err(|_| GameError::SendFailed("Could not query run status".to_owned()))?;

            match run.status {
                RunStatus::Completed => {
                    let query = [("limit", "1")];
                    let response = self
                        .client
                        .threads()
                        .messages(&self.thread_id)
                        .list(&query)
                        .await
                        .map_err(|_| {
                            GameError::SendFailed("Could not retrieve response".to_owned())
                        })?;
                    let message_id = response.data.first().unwrap().id.clone();
                    let message = self
                        .client
                        .threads()
                        .messages(&self.thread_id)
                        .retrieve(&message_id)
                        .await
                        .map_err(|_| {
                            GameError::SendFailed("Could not retrieve message".to_owned())
                        })?;
                    let content = message
                        .content
                        .first()
                        .ok_or(GameError::SendFailed("No messages in response".to_owned()))?;

                    debug!("Received: {:?}", &content);

                    let output = match content {
                        MessageContent::Text(text) => serde_json::from_str(&text.text.value)
                            .map_err(|json_err| {
                                GameError::UnexpectedResponse(format!("JSON error: {}", json_err))
                            }),
                        MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                            Err(GameError::UnexpectedResponse("Received image".to_owned()))
                        }
                        MessageContent::Refusal(refusal) => {
                            Err(GameError::RefusalResponse(refusal.refusal.clone()))
                        }
                    };

                    if let Err(ref error) = output {
                        error!("{:?}", error);
                    }

                    return output;
                }
                RunStatus::Failed => return Err(GameError::SendFailed("Run failed".to_owned())),
                _ => (),
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

const AI_NAME: &'static str = "uQuest GM";
const AI_MODEL: &'static str = "gpt-4o";
const AI_RESPONSE_DESC: &'static str = "A series of updates to the game state, including text to be output to the user. A QuestDefinition is sent as a response to a Start message.";
const AI_INST: &'static str = "You are the game master for a text-based adventure game. You will run a session containing a simple quest for a single player character. You must not take any actions on behalf of the player character, the player character has full control over what they do. Suggest some possible actions to the user in each description. You will receive commands in JSON format according to the following schema:\n\n";
const AI_INST_PROLOGUE: &'static str = "\n\nYou may respond to a command with multiple different 'updates'. Only the Description update will be presented to the user, so any description or dialogue intended for the user must be in a Description update.";
