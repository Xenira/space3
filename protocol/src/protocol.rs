use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Protocol {
    // Misc
    STATUS_RESPONSE(Status),
    EMPTY(String),

    // User
    REGISTRATION_REQUEST(Credentials),
    LOGIN_REQUEST(Credentials),
    LOGIN_RESPONSE(LoginResponse),
    USER_RESPONSE(UserData),

    // Lobby
    LOBBY_JOIN_REQUEST(LobbyJoinRequest),
    LOBBY_STATUS_RESPONSE(LobbyInfo),
    SET_HEADER_RESPONSE(Header),

    // Error
    NETWORKING_ERROR(Error),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    pub username: String,
    pub currency: i32,
    pub lobby: Option<LobbyInfo>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LobbyJoinRequest {
    pub name: String,
    pub passphrase: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LobbyInfo {
    pub name: String,
    pub users: Vec<LobbyUser>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LobbyUser {
    pub name: String,
    pub ready: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub key: String,
    pub user: UserData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub message: String,
    pub status: u16,
}

impl Error {
    pub fn new(status: u16, message: String) -> Error {
        Error { message, status }
    }

    pub fn new_protocol(status: u16, message: String) -> Protocol {
        Protocol::NETWORKING_ERROR(Error::new(status, message))
    }
}
