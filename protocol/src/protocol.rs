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
    GameShopResponse(bool, Vec<Option<(u8, CharacterInstance)>>),
    BuyRequest(BuyRequest),
    RerollShopRequest,
    BuyResponse(
        GameUserInfo,
        Vec<Option<(u8, CharacterInstance)>>,
        Vec<Option<CharacterInstance>>,
    ),
    SellResponse(GameUserInfo, Vec<Option<CharacterInstance>>),
    GameBattleResponse(BattleResponse),
    GameBattleResultResponse(BattleResult),
    GameEndResponse(GameResult),
    GameUserInfoResponse(GameUserInfo),

    CharacterMoveRequest,
    BoardResponse(Vec<Option<CharacterInstance>>),

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
    pub actions: Vec<BattleAction>,
    pub start_own: Vec<Option<CharacterInstance>>,
    pub start_opponent: Vec<Option<CharacterInstance>>,
}

impl BattleResponse {
    pub fn swap_players(&self) -> Self {
        Self {
            actions: self
                .actions
                .iter()
                .map(|a| a.swap_players())
                .collect::<Vec<_>>(),
            start_own: self.start_opponent.clone(),
            start_opponent: self.start_own.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleAction {
    pub action: BattleActionType,
    pub source: i32,
    pub target: Option<i32>,
    pub result_own: Vec<Option<CharacterInstance>>,
    pub result_opponent: Vec<Option<CharacterInstance>>,
}

impl BattleAction {
    pub fn swap_players(&self) -> Self {
        let mut result = self.clone();

        let res_op = result.result_opponent;
        result.result_opponent = result.result_own;
        result.result_own = res_op;

        result
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BattleActionType {
    Attack,
    Die,
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
pub struct GameUserInfo {
    pub name: String,
    pub experience: i32,
    pub health: i32,
    pub money: i32,
    pub avatar: Option<i32>,
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
pub struct CharacterInstance {
    pub id: i32,
    pub character_id: i32,
    pub position: i32,
    pub upgraded: bool,
    pub attack: i32,
    pub defense: i32,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
}

impl CharacterInstance {
    pub fn from(character: &Character, upgraded: bool) -> Self {
        let (upgrade_atk, upgrade_hp) = if let Some(upgrade) = &character.upgrade {
            (upgrade.attack, upgrade.health)
        } else {
            (0, 0)
        };

        Self {
            id: -1,
            character_id: character.id,
            position: -1,
            upgraded: upgraded,
            attack: if upgraded {
                upgrade_atk
            } else {
                character.attack
            },
            defense: if upgraded {
                upgrade_hp
            } else {
                character.health
            },
            attack_bonus: 0,
            defense_bonus: 0,
        }
    }

    pub fn with_position(mut self, position: i32) -> Self {
        self.position = position;
        self
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Error {
    pub message: String,
    pub status: u16,
    pub reference: Option<Box<Protocol>>,
}

impl Error {
    pub fn new(status: u16, message: String) -> Self {
        Self {
            message,
            status,
            ..Default::default()
        }
    }

    pub fn new_protocol(status: u16, message: String) -> Protocol {
        Protocol::NetworkingError(Self::new(status, message))
    }

    pub fn new_protocol_response(status: u16, message: String, reference: Protocol) -> Protocol {
        Protocol::NetworkingError(Self {
            message,
            status,
            reference: Some(Box::new(reference)),
        })
    }

    pub fn with_reference(mut self, reference: Protocol) -> Self {
        self.reference = Some(Box::new(reference));
        self
    }
}
