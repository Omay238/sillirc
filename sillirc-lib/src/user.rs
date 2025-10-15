use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(default)]
pub struct User {
    username: String,
    uuid: Uuid,
}
impl Default for User {
    fn default() -> Self {
        Self {
            username: String::new(),
            uuid: Uuid::new_v4(),
        }
    }
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            uuid: Uuid::new_v4(),
        }
    }
}
