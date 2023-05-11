use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::Duration;
use rocket::State;

use crate::model::users::User;

struct ActiveUsers {
    // users: HashMap<String, Arc<Notify>>,
}

// #[get("/poll")]
// async fn poll(user: &User, active_polls: State<Mutex<ActiveUsers>>) {
//     todo!()
//     // match notif_opt {
//     //     Some(notification) => {
//     //         match async_std::future::timeout(Duration::from_secs(seconds), notification.notified())
//     //             .await
//     //         {
//     //             Ok(_) => "Done".to_string(),
//     //             Err(_) => "Timeout".to_string(),
//     //         }
//     //     }
//     //     None => return "Who are you?".to_string(),
//     // }
// }
