use async_channel::{Receiver, SendError, Sender};
use async_std::future;
use protocol::protocol::{Error, Protocol};
use rocket::futures::future::join_all;
use rocket::serde::json::Json;
use static_init::dynamic;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use crate::model::users::User;

type NotificationChannels = Arc<(Sender<Protocol>, Receiver<Protocol>)>;
#[derive(Default)]
pub struct ActivePolls {
    polls: HashMap<i32, NotificationChannels>,
    channels: HashMap<Channel, HashSet<i32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Lobby(i32),
    Game(Uuid),
}

#[dynamic]
static ACTIVE_POLLS: Mutex<ActivePolls> = Mutex::new(ActivePolls::default());

impl ActivePolls {
    pub fn get() -> &'static Mutex<Self> {
        &ACTIVE_POLLS
    }

    pub fn register(user: &User) -> NotificationChannels {
        let channel = {
            let polls = &Self::get().lock().unwrap().polls;
            polls.get(&user.id).cloned()
        };

        match channel {
            Some(c) => c,
            None => {
                let new_channel = Arc::new(async_channel::unbounded());
                Self::get()
                    .lock()
                    .unwrap()
                    .polls
                    .insert(user.id, new_channel.clone());
                new_channel
            }
        }
    }

    pub async fn notify(user: i32, data: Protocol) -> Result<(), SendError<Protocol>> {
        let channel = {
            let polls = &Self::get().lock().unwrap().polls;
            polls.get(&user).cloned()
        };

        let channel = match channel {
            Some(channel) => channel,
            None => {
                let channel = Arc::new(async_channel::unbounded());
                Self::get()
                    .lock()
                    .unwrap()
                    .polls
                    .insert(user, channel.clone());
                channel
            }
        };

        channel.0.send(data).await
    }

    pub async fn notify_channel(
        channel: &Channel,
        data: Protocol,
    ) -> Vec<Result<(), SendError<Protocol>>> {
        let users = {
            let polls = &Self::get().lock().unwrap().channels;
            polls.get(channel).cloned()
        };
        if let Some(users) = users {
            return join_all(
                users
                    .iter()
                    .map(|user| ActivePolls::notify(*user, data.clone())),
            )
            .await;
        }
        Vec::new()
    }

    fn join(&mut self, channel: Channel, user: i32) {
        if let Some(users) = self.channels.get_mut(&channel) {
            users.insert(user);
        } else {
            let mut users = HashSet::new();
            users.insert(user);
            self.channels.insert(channel, users);
        }
    }

    pub fn join_user(channel: Channel, user: i32) {
        let mut polls = Self::get().lock().unwrap();
        polls.join(channel, user);
    }

    pub fn join_users(channel: Channel, users: Vec<i32>) {
        let mut polls = Self::get().lock().unwrap();
        for user in users {
            polls.join(channel, user);
        }
    }

    pub fn leave_channel(channel: &Channel, user: &i32) {
        let mut polls = Self::get().lock().unwrap();
        if let Some(users) = polls.channels.get_mut(channel) {
            users.remove(user);
        }
    }

    pub fn close_channel(channel: &Channel) {
        let mut polls = Self::get().lock().unwrap();
        polls.channels.remove(channel);
    }
}

#[get("/poll")]
pub async fn poll(user: &User) -> Json<Protocol> {
    let notify = ActivePolls::register(user);

    let dur = Duration::from_secs(30);

    match future::timeout(dur, notify.1.recv()).await {
        Ok(res) => match res {
            Ok(res) => Json(res),
            Err(err) => Json(Error::new_protocol(500, err.to_string())),
        },
        Err(_) => Json(Protocol::PollingTimeout),
    }
}
