use crate::{
    game::game_instance::GameInstance,
    model::lobbies::Lobby,
    schema::lobbies,
    service::{game_service, lobby_service},
    Database,
};
use diesel::{dsl::now, prelude::*, ExpressionMethods, QueryDsl};
use rocket::{
    log::private::{debug, trace, warn},
    tokio::{self, sync::Mutex},
};
use std::{borrow::BorrowMut, collections::HashMap, sync::Arc, time::Duration};
use uuid::Uuid;

type GameMap = HashMap<Uuid, Arc<Mutex<GameInstance>>>;

pub async fn long_running_task(db: Database, games: &Arc<Mutex<GameMap>>) {
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
                let game = game_service::start_game(&db, &lobby).await;
                games
                    .lock()
                    .await
                    .insert(game.game_id, Arc::new(Mutex::new(game)));
                db.run(move |con| {
                    lobby_service::close_lobby(con, &lobby);
                })
                .await;
            }
        } else {
            warn!("Failed to load lobbies to start")
        }

        for game in games.lock().await.values_mut() {
            if !(game.lock().await).turn.is_next() {
                continue;
            }

            debug!("Next turn for game {:?}", game.lock().await.game_id);
            if game_service::next_turn(game.lock().await.borrow_mut()).await {
                debug!(
                    "Game {:?} is over, removing from active games list",
                    game.lock().await.game_id
                );
                games.lock().await.remove(&game.lock().await.game_id);
            }
            trace!("Done processing game {:?}", game.lock().await.game_id);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
