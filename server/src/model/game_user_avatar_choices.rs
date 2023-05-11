use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{delete, dsl::count_star, prelude::*, update};
use protocol::protocol::Protocol;

use crate::{
    model::game_users::GameUser,
    schema::{game_user_avatar_choices, game_users, games},
    service::game_service::notify_users,
    Database,
};
use protocol::gods::GODS;
use rocket::{http::Status, log::private::debug, serde::json::Json};

use super::{game::Game, game_users::GameUserUpdate};

#[derive(Identifiable, Queryable, Associations, Clone, Debug)]
#[diesel(belongs_to(Game))]
#[diesel(belongs_to(GameUser))]
pub struct GameUserAvatarChoice {
    pub id: i32,
    pub game_id: i32,
    pub game_user_id: i32,
    pub avatar_id: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "game_user_avatar_choices"]
pub struct NewGameUserAvatarChoice {
    pub game_id: i32,
    pub game_user_id: i32,
    pub avatar_id: i32,
}

impl NewGameUserAvatarChoice {
    pub fn from_parents(game_id: i32, game_user_id: i32, avatar_id: i32) -> Self {
        Self {
            game_id,
            game_user_id,
            avatar_id,
        }
    }
}

pub enum GameUserAvatarChoiceError {
    AvatarNotAvailable,
    AvatarAlreadyChosen,
}

#[put("/games/avatar/<avatar_id>")]
pub async fn select_avatar(
    db: Database,
    mut game: Game,
    game_user: GameUser,
    avatar_id: i32,
) -> Json<Protocol> {
    let (choice, game) = db
        .run(move |c| {
            let choosen_avatar = GameUserAvatarChoice::belonging_to(&game_user)
                .filter(game_user_avatar_choices::avatar_id.eq(avatar_id))
                .first::<GameUserAvatarChoice>(c);

            // TODO: Check if avatar is needed in if let
            if let Ok(_) = choosen_avatar {
                // TODO: Handle result
                update(game_users::table)
                    .set(game_users::avatar_id.eq(avatar_id))
                    .filter(game_users::id.eq(game_user.id))
                    .execute(c)
                    .unwrap();

                delete(game_user_avatar_choices::table)
                    .filter(game_user_avatar_choices::game_user_id.eq(game_user.id))
                    .execute(c)
                    .unwrap();

                if game
                    .next_battle
                    .map(|next| {
                        DateTime::<Utc>::from_utc(next, Utc)
                            .signed_duration_since(Utc::now())
                            .num_seconds()
                    })
                    .map_or(false, |seconds| seconds > 5)
                {
                    if let Ok(choices_left) = GameUserAvatarChoice::belonging_to(&game)
                        .select(count_star())
                        .first::<i64>(c)
                    {
                        if choices_left == 0 {
                            debug!(
                                "All players have chosen their avatar, starting game in 5 seconds"
                            );
                            game.next_battle = Some(
                                Utc::now()
                                    .naive_utc()
                                    .checked_add_signed(chrono::Duration::seconds(5))
                                    .unwrap(),
                            );
                            update(games::table)
                                .set(games::next_battle.eq(game.next_battle))
                                .filter(games::id.eq(game.id))
                                .execute(c);
                        }
                    }
                }

                (
                    Protocol::AvatarSelectResponse(GODS[avatar_id as usize].clone()),
                    game,
                )
            } else {
                (
                    protocol::protocol::Error::new_protocol(
                        Status::Conflict.code,
                        "Avatar not available".to_string(),
                    ),
                    game,
                )
            }
        })
        .await;

    notify_users(&game).await;
    Json(choice)
}
