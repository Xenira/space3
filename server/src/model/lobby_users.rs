use crate::schema::lobby_users;
use chrono::NaiveDateTime;
use protocol::protocol;

use super::{lobbys::Lobby, users::User};

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Lobby)]
#[belongs_to(User)]
#[table_name = "lobby_users"]
pub struct LobbyUser {
    id: i32,
    lobby_id: i32,
    user_id: i32,
    username: String,
    ready: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Into<protocol::LobbyUser> for LobbyUser {
    fn into(self) -> protocol::LobbyUser {
        protocol::LobbyUser {
            name: self.username,
            ready: self.ready,
        }
    }
}

#[derive(Insertable)]
#[table_name = "lobby_users"]
pub struct NewLobbyUser {
    lobby_id: i32,
    user_id: i32,
    username: String,
}

impl NewLobbyUser {
    pub fn from_parents(lobby: &Lobby, user: &User) -> Self {
        Self {
            lobby_id: lobby.id,
            user_id: user.id,
            username: user.username.to_string(),
        }
    }
}
