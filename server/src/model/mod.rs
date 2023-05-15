pub(crate) mod game;
pub(crate) mod game_user_avatar_choices;
pub(crate) mod game_user_characters;
pub(crate) mod game_users;
pub mod lobbies;
pub mod lobby_users;
pub mod polling;
pub(crate) mod shop;
pub mod users;

pub mod model {
    use protocol::protocol::{Protocol, Status};
    use rocket::{serde::json::Json, Route};

    use super::{game_user_avatar_choices, lobbies, polling, shop, users};

    #[get("/status")]
    fn status() -> Json<Protocol> {
        Json(Protocol::StatusResponse(Status {
            version: "v0.1".to_string(),
        }))
    }

    pub fn get_api() -> Vec<Route> {
        routes![
            status,
            users::register,
            users::login,
            users::me,
            lobbies::get_current_loby_info,
            lobbies::join_lobby,
            lobbies::leave_lobby,
            lobbies::toggle_ready_state,
            lobbies::start_lobby_timer,
            lobbies::stop_lobby_timer,
            game_user_avatar_choices::select_avatar,
            shop::get_shop,
            shop::reroll_shop,
            shop::buy_character,
            polling::poll,
            polling::notify,
        ]
    }
}
