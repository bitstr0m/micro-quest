use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct PlayerCharacterBuilder {
    name: String,
    race: String,
    class: String,
}

impl PlayerCharacterBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            race: "human".to_owned(),
            class: "fighter".to_owned(),
        }
    }

    pub fn with_race(mut self, race: String) -> Self {
        self.race = race;
        self
    }

    pub fn with_class(mut self, class: String) -> Self {
        self.class = class;
        self
    }

    pub fn build(self) -> PlayerCharacter {
        PlayerCharacter {
            name: self.name,
            race: self.race,
            class: self.class,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlayerCharacter {
    name: String,
    race: String,
    class: String,
}

impl PlayerCharacter {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn race(&self) -> &str {
        &self.race
    }

    pub fn class(&self) -> &str {
        &self.class
    }
}
