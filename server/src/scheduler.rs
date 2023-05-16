use std::{future::Future, pin::Pin, time::Duration};

use diesel::{dsl::now, prelude::*, ExpressionMethods, QueryDsl};
use rocket::{log::private::warn, tokio};

use crate::{
    model::{game::Game, lobbies::Lobby},
    schema::{games, lobbies},
    service::{game_service, lobby_service},
    Database,
};

pub async fn long_running_task(db: Database) {
    loop {
        // trace!("Long running task");
        if let Ok(lobbies) = db
            .run(move |con| {
                lobbies::table
                    .filter(lobbies::start_at.le(now))
                    .load::<Lobby>(con)
            })
            .await
        {
            for lobby in lobbies {
                debug!("Starting lobby {:?}", lobby);
                game_service::start_game(&db, &lobby).await;
                db.run(move |con| {
                    lobby_service::close_lobby(con, &lobby);
                })
                .await;
            }
        } else {
            warn!("Failed to load lobbies to start")
        }

        if let Ok(games) = db
            .run(move |con| {
                games::table
                    .filter(games::next_battle.le(now))
                    .load::<Game>(con)
            })
            .await
        {
            for game in games {
                debug!("Next turn for game {:?}", game);
                game_service::next_turn(&db, &game).await;
            }
        } else {
            warn!("Failed to load games")
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
