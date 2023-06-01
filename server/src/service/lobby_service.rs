use crate::{
    model::{
        lobbies::{Lobby, LobbyError, LobbyWithUsers, NewLobby},
        lobby_users::{LobbyUser, NewLobbyUser},
        polling::{ActivePolls, Channel},
        users::User,
    },
    schema::{lobbies, lobby_users},
    Database,
};

use chrono::NaiveDateTime;
use diesel::{
    delete,
    dsl::{count, exists, not},
    insert_into,
    prelude::*,
    update,
};
use protocol::protocol::{LobbyJoinRequest, Protocol};
use rocket::log::private::{debug, trace, warn};

pub async fn join_lobby(
    db: &Database,
    lobby: LobbyJoinRequest,
    user: &User,
) -> Result<(), LobbyError> {
    let user = user.clone();
    let user_id = user.id;
    let lobby_name = lobby.name.clone();
    if let Ok(lobby) = db
        .run(move |con| {
            let lobby = get_or_create_lobby(con, &lobby, user_id);
            if let Ok(lobby) = lobby {
                debug!("Target lobby {:?} for user {:?}", lobby, user);
                debug!("Deleting old lobby user entries for user {:?}", user);
                if let Err(err) = delete(lobby_users::table)
                    .filter(lobby_users::user_id.eq(user.id))
                    .execute(con)
                {
                    error!("Failed to delete lobby user entries {:?}", err);
                    return Err(LobbyError::Internal);
                }

                debug!("Inserting new lobby user entry for user {:?}", user);
                if let Err(err) = insert_into(lobby_users::table)
                    .values::<NewLobbyUser>(NewLobbyUser::from_parents(&lobby, &user))
                    .execute(con)
                {
                    error!("Failed to create lobby user entry {:?}", err);
                    return Err(LobbyError::Internal);
                }

                ActivePolls::join_user(Channel::Lobby(lobby.id), user.id);
                Ok(lobby)
            } else {
                warn!("Failed to get or create lobby {:?}", lobby);
                Err(LobbyError::Internal)
            }
        })
        .await
    {
        debug!("Joined lobby {:?}. Sending update.", lobby);
        notify_lobby_users(db, lobby.id).await;
        Ok(())
    } else {
        warn!("Failed to join lobby {:?}", lobby_name);
        Err(LobbyError::Internal)
    }
}

fn get_or_create_lobby(
    con: &mut PgConnection,
    lobby: &LobbyJoinRequest,
    master_id: i32,
) -> Result<Lobby, LobbyError> {
    debug!("Getting or creating lobby {:?}", lobby);
    if let Ok(existing_lobby) = lobbies::table
        .filter(lobbies::name.eq(&lobby.name))
        .filter(lobbies::passphrase.eq(&lobby.passphrase))
        .first::<Lobby>(con)
    {
        trace!("Found existing lobby {:?}", existing_lobby);
        match LobbyUser::belonging_to(&existing_lobby)
            .select(count(lobby_users::id))
            .first::<i64>(con)
        {
            Ok(player_count) if player_count >= 8 => Err(LobbyError::Full),
            _ => Ok(existing_lobby),
        }
    } else {
        debug!("Creating new lobby {:?}", lobby);
        match insert_into(lobbies::table)
            .values::<NewLobby>(NewLobby::from_join_request(lobby, master_id))
            .get_results::<Lobby>(con)
        {
            Ok(results) => {
                debug!("Created new lobby {:?}", results.first().unwrap());
                results.first().cloned().ok_or(LobbyError::Internal)
            }
            Err(e) => {
                let new_lobby: NewLobby = NewLobby::from_join_request(lobby, master_id);
                warn!("Failed to create lobby {:?}: {}", new_lobby, e);
                Err(LobbyError::Internal)
            }
        }
    }
}

pub async fn remove_user_from_lobbies(db: &Database, user: &User) {
    debug!("Removing user {:?} from lobbies", user);
    let user_id = user.id;
    if let Ok(lobby_ids) = db
        .run(move |con| {
            delete(lobby_users::table)
                .filter(lobby_users::user_id.eq(user_id))
                .returning(lobby_users::lobby_id)
                .get_results(con)
        })
        .await
    {
        reassign_master(db).await;

        for lobby_id in lobby_ids {
            notify_lobby_users(db, lobby_id).await;
        }
    }
}

pub async fn set_ready_state(db: &Database, user: &LobbyUser, rdy: bool) {
    let user_id = user.id;
    db.run(move |con| {
        diesel::update(lobby_users::table)
            .filter(lobby_users::id.eq(user_id))
            .set(lobby_users::ready.eq(rdy))
            .execute(con)
            .unwrap();
    })
    .await;

    notify_lobby_users(db, user.lobby_id).await;
}

pub async fn start_lobby_timer(db: &Database, lobby: &LobbyWithUsers) {
    let users_rdy = !lobby.users.iter().any(|u| !u.ready);

    let start_time =
        chrono::Utc::now().naive_utc() + chrono::Duration::seconds(if users_rdy { 5 } else { 20 });

    db.run(move |con| {
        update(lobbies::table)
            .set(lobbies::start_at.eq(Some(start_time)))
            .execute(con)
            .unwrap()
    })
    .await;

    notify_lobby_users(db, lobby.lobby.id).await;
}

pub async fn stop_lobby_timer(db: &Database, lobby: i32) {
    db.run(move |con| {
        update(lobbies::table)
            .set(lobbies::start_at.eq(None::<NaiveDateTime>))
            .execute(con)
            .unwrap()
    })
    .await;

    notify_lobby_users(db, lobby).await;
}

async fn reassign_master(db: &Database) {
    let lobby_ids = db
        .run(move |con| {
            lobbies::table
                .left_join(
                    lobby_users::table.on(lobby_users::user_id
                        .eq(lobbies::master_id)
                        .and(lobby_users::lobby_id.eq(lobbies::id))),
                )
                .filter(lobby_users::id.is_null())
                .select(lobbies::all_columns)
                .load::<Lobby>(con)
                .unwrap()
                .iter()
                .filter_map(|lobby| {
                    if let Ok(user) = LobbyUser::belonging_to(lobby)
                        .select(lobby_users::user_id)
                        .first::<i32>(con)
                    {
                        if let Err(err) = update(lobbies::table)
                            .filter(lobbies::id.eq(lobby.id))
                            .set(lobbies::master_id.eq(user))
                            .execute(con)
                        {
                            error!("Failed to update lobby master {:?}", err);
                            None
                        } else {
                            Some(lobby.id)
                        }
                    } else {
                        remove_empty_lobbies(con);
                        None
                    }
                })
                .collect::<Vec<i32>>()
        })
        .await;

    for lobby in lobby_ids {
        notify_lobby_users(db, lobby).await;
    }
}

fn remove_empty_lobbies(con: &mut PgConnection) {
    debug!("Removing empty lobbies");
    if let Err(err) = delete(lobbies::table)
        .filter(not(exists(
            lobby_users::table.select(lobby_users::id).distinct(),
        )))
        .execute(con)
    {
        error!("Failed to delete empty lobbies {}", err)
    }
}

pub fn close_lobby(con: &mut PgConnection, lobby: &Lobby) -> Result<(), String> {
    debug!("Closing lobby {:?}", lobby);
    let result = delete(lobbies::table)
        .filter(lobbies::id.eq(lobby.id))
        .execute(con);

    if result.is_ok() {
        Ok(())
    } else {
        Err("Failed to delete lobby".to_string())
    }
}

pub async fn notify_lobby_users(db: &Database, lobby: i32) -> Result<(), String> {
    debug!("Sending change notification for lobby {:?}", lobby);
    if let Ok((lobby, users)) = db
        .run(move |con| {
            lobbies::table
                .filter(lobbies::id.eq(lobby))
                .first::<Lobby>(con)
                .map(|lobby| {
                    let users = LobbyUser::belonging_to(&lobby)
                        .load(con)
                        .unwrap_or_default();
                    (lobby, users)
                })
        })
        .await
    {
        ActivePolls::notify_channel(
            &Channel::Lobby(lobby.id),
            Protocol::LobbyStatusResponse(lobby.into_lobby_info(&users)),
        )
        .await;
        Ok(())
    } else {
        Err("Failed to notify lobby users".to_string())
    }
}
