pub mod users;

pub mod model {
    use protocol::protocol::{Protocol, Status};
    use rocket::{serde::json::Json, Route};

    use super::users;

    #[get("/status")]
    fn status() -> Json<Protocol> {
        Json(Protocol::STATUS_RESPONSE(Status {
            version: "v0.1".to_string(),
        }))
    }

    pub fn get_api() -> Vec<Route> {
        routes![status, users::register, users::login, users::me]
    }
}
