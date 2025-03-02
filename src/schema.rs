use schemars::{schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::character::PlayerCharacter;

// TODO Do we need Serialize on data we only get sent from the LLM?

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub enum AIInput {
    Start(PlayerCharacter),
    UserInput(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AIOutput {
    pub updates: Vec<QuestUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub enum QuestUpdate {
    QuestDefinition(QuestDefinition),
    Description(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestDefinition {
    // Or QuestManifest?
    pub title: String,
    pub description: String,
    pub objective_summary: String,
}
