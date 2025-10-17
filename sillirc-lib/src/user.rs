use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(default)]
pub struct User {
    username: String,
    uuid: Uuid,
    color: (u8, u8, u8),
}

impl User {
    pub fn new(username: String) -> Self {
        let uuid = Uuid::new_v4();
        let mut uuid_iter = uuid.as_bytes().iter();
        Self {
            username,
            uuid,
            color: (
                *uuid_iter.next().expect(""),
                *uuid_iter.next().expect(""),
                *uuid_iter.next().expect(""),
            ),
        }
    }

    pub fn is_unnamed(&self) -> bool {
        self.username.is_empty()
    }

    pub fn set_username(self, new_username: String) -> Self {
        Self {
            username: new_username,
            uuid: self.uuid,
            color: self.color,
        }
    }

    pub fn set_color(self, new_color: (u8, u8, u8)) -> Self {
        Self {
            username: self.username,
            uuid: self.uuid,
            color: new_color,
        }
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        self.color
    }
}

impl Default for User {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        let mut uuid_iter = uuid.as_bytes().iter();
        Self {
            username: String::new(),
            uuid,
            color: (
                *uuid_iter.next().expect(""),
                *uuid_iter.next().expect(""),
                *uuid_iter.next().expect(""),
            ),
        }
    }
}
