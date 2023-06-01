use chrono::{DateTime, Duration, Utc};
use protocol_types::{character::Character, heros::God, prelude::Ability};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const START_TIME: i64 = 45;
const EXP_PER_LEVEL: u8 = 3;

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
    DisplaynameResponse(String),

    // Lobby
    LobbyJoinRequest(LobbyJoinRequest),
    LobbyStatusResponse(LobbyInfo),
    LobbyLeaveResponse,
    LobbyStartResponse,
    LobbyKickResponse,

    // Game
    // TODO: Change to [God; 4]
    GameUpdateResponse(GameUpdate),
    GameStartResponse([i32; 4]),
    AvatarSelectResponse(God),
    GameShopResponse(GameUserInfo, bool, Vec<Option<CharacterInstance>>),
    BuyRequest(BuyRequest),
    RerollShopRequest,
    BuyResponse(
        GameUserInfo,
        Vec<Option<CharacterInstance>>,
        Vec<Option<CharacterInstance>>,
    ),
    SellResponse(GameUserInfo, Vec<Option<CharacterInstance>>),
    GameBattleResponse(BattleResponse),
    GameBattleResultResponse(BattleResult),
    GameEndResponse(GameResult),
    GameUserInfoResponse(GameUserInfo),
    GameUsersResponse(Vec<GameOpponentInfo>),

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
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
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
    pub opponent: GameOpponentInfo,
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
            opponent: self.opponent.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleAction {
    pub action: BattleActionType,
    pub source: Uuid,
    pub target: Option<Uuid>,
    pub result_own: Vec<Option<CharacterInstance>>,
    pub result_opponent: Vec<Option<CharacterInstance>>,
}

impl BattleAction {
    pub fn swap_players(&self) -> Self {
        let mut result = self.clone();

        std::mem::swap(&mut result.result_opponent, &mut result.result_own);

        result
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BattleActionType {
    Attack,
    Die,
    Ability,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleResult {
    pub dmg: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUpdate {
    pub turn: Turn,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUserInfo {
    pub name: String,
    pub experience: u8,
    pub health: i16,
    pub money: u16,
    pub avatar: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameOpponentInfo {
    pub name: String,
    pub experience: u8,
    pub health: i16,
    pub character_id: i32,
    pub is_next_opponent: bool,
}

impl GameOpponentInfo {
    pub fn get_lvl(&self) -> u8 {
        self.experience / EXP_PER_LEVEL
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameResult {
    pub place: u8,
    pub reward: i32,
    pub ranking: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuyRequest {
    pub character_idx: u8,
    pub target_idx: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterInstance {
    pub id: Uuid,
    pub character_id: i32,
    pub position: i32,
    pub upgraded: bool,
    pub attack: i32,
    pub health: i32,
    pub attack_bonus: i32,
    pub health_bonus: i32,
    pub cost: u8,
    pub abilities: Vec<Ability>,
}

impl CharacterInstance {
    pub fn from(character: &Character, upgraded: bool) -> Self {
        let (upgrade_atk, upgrade_hp, upgrade_abilities) = if let Some(upgrade) = &character.upgrade
        {
            (upgrade.attack, upgrade.health, upgrade.abilities.clone())
        } else {
            (0, 0, vec![])
        };

        Self {
            id: Uuid::new_v4(),
            character_id: character.id,
            position: -1,
            upgraded,
            attack: if upgraded {
                upgrade_atk
            } else {
                character.attack
            },
            health: if upgraded {
                upgrade_hp
            } else {
                character.health
            },
            attack_bonus: 0,
            health_bonus: 0,
            cost: character.cost,
            abilities: if upgraded {
                upgrade_abilities
            } else {
                character.abilities.clone()
            },
        }
    }

    pub fn with_position(mut self, position: i32) -> Self {
        self.position = position;
        self
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_attack_bonus(mut self, attack_bonus: i32) -> Self {
        self.attack_bonus = attack_bonus;
        self
    }

    pub fn with_health_bonus(mut self, health_bonus: i32) -> Self {
        self.health_bonus = health_bonus;
        self
    }

    pub fn get_total_attack(&self) -> i32 {
        self.attack + self.attack_bonus
    }

    pub fn get_total_health(&self) -> i32 {
        self.health + self.health_bonus
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Shop(u16, DateTime<Utc>),
    Combat(u16, DateTime<Utc>),
}

impl Default for Turn {
    fn default() -> Self {
        Self::Combat(0, Utc::now() + Duration::seconds(START_TIME))
    }
}

impl From<Turn> for u16 {
    fn from(val: Turn) -> Self {
        match val {
            Turn::Shop(turn, _) => turn,
            Turn::Combat(turn, _) => turn,
        }
    }
}

impl From<Turn> for DateTime<Utc> {
    fn from(val: Turn) -> Self {
        match val {
            Turn::Shop(_, turn_time) => turn_time,
            Turn::Combat(_, turn_time) => turn_time,
        }
    }
}

impl Turn {
    pub fn next(&mut self, next_turn: DateTime<Utc>) {
        match self {
            Self::Shop(turn, _) => *self = Self::Combat(*turn, next_turn),
            Self::Combat(turn, _) => *self = Self::Shop(*turn + 1, next_turn),
        }
    }

    pub fn is_next(&self) -> bool {
        match self {
            Self::Shop(_, turn_time) => Utc::now() >= *turn_time,
            Self::Combat(_, turn_time) => Utc::now() >= *turn_time,
        }
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
