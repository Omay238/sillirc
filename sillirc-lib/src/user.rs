use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)]
pub struct User {
    username: String,
    uuid: Uuid,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn is_unnamed(&self) -> bool {
        self.username.is_empty()
    }

    pub fn set_username(&self, new_username: String) -> Self {
        Self {
            username: new_username,
            uuid: self.uuid,
        }
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            username: String::new(),
            uuid: Uuid::new_v4(),
        }
    }
}
