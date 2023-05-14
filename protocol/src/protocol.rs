use chrono::{DateTime, Utc};
use protocol_types::{character::Character, heros::God};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Protocol {
    // Misc
    StatusResponse(Status),
    EMPTY(String),

    // User
    RegistrationRequest(Credentials),
    LoginRequest(Credentials),
    LoginResponse(LoginResponse),
    UserResponse(UserData),

    // Lobby
    LobbyJoinRequest(LobbyJoinRequest),
    LobbyStatusResponse(LobbyInfo),
    LobbyLeaveResponse,
    LobbyStartResponse,
    LobbyKickResponse,

    // Game
    // TODO: Change to [God; 4]
    GameUpdateResponse(GameUpdate),
    GameStartResponse(Vec<God>),
    AvatarSelectResponse(God),
    GameShopResponse(Vec<Character>),
    GameBattleResponse(BattleResponse),
    GameBattleResultResponse(BattleResult),
    GameEndResponse(GameResult),

    // Polling
    PollingTimeout,

    // Error
    NetworkingError(Error),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    pub username: String,
    pub currency: i32,
    pub lobby: Option<LobbyInfo>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LobbyJoinRequest {
    pub name: String,
    pub passphrase: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LobbyInfo {
    pub name: String,
    pub users: Vec<LobbyUser>,
    pub master: i32,
    pub start_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct LobbyUser {
    pub id: i32,
    pub name: String,
    pub ready: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginResponse {
    pub key: String,
    pub user: UserData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleResponse {
    pub opponent: Character,
    pub actions: Vec<BattleAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleAction {
    pub action: String,
    pub target: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleResult {
    pub dmg: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUpdate {
    pub turn: i32,
    pub next_turn_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameResult {
    pub place: i32,
    pub reward: i32,
    pub ranking: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuyRequest {
    pub character_idx: u8,
    pub target_idx: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    pub message: String,
    pub status: u16,
}

impl Error {
    pub fn new(status: u16, message: String) -> Error {
        Error { message, status }
    }

    pub fn new_protocol(status: u16, message: String) -> Protocol {
        Protocol::NetworkingError(Error::new(status, message))
    }
}
