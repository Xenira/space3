pub mod lobby_users;
pub mod lobbys;
pub mod users;

pub mod model {
    use protocol::protocol::{Protocol, Status};
    use rocket::{serde::json::Json, Route};

    use super::{lobbys, users};

    #[get("/status")]
    fn status() -> Json<Protocol> {
        Json(Protocol::STATUS_RESPONSE(Status {
            version: "v0.1".to_string(),
        }))
    }

    pub fn get_api() -> Vec<Route> {
        routes![
            status,
            users::register,
            users::login,
            users::me,
            lobbys::get_current_loby_info,
            lobbys::join_lobby
        ]
    }
}
