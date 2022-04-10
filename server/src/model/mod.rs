pub mod users;

pub mod model {
    use rocket::Route;

    use super::users;

    pub fn get_api() -> Vec<Route> {
        routes![users::register, users::login]
    }
}
