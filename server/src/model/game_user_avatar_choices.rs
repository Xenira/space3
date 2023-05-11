use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{delete, dsl::count_star, prelude::*, update};

use crate::{
    model::game_users::GameUser,
    schema::{game_user_avatar_choices, game_users, games},
    Database,
};
use rocket::{http::uri::Path, request::FromRequest, Request};

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
pub async fn select_avatar(db: Database, game: Game, game_user: GameUser, avatar_id: i32) {
    db.run(move |c| {
        let choosen_avatar = GameUserAvatarChoice::belonging_to(&game_user)
            .filter(game_user_avatar_choices::avatar_id.eq(avatar_id))
            .first::<GameUserAvatarChoice>(c);

        update(game_users::table)
            .set(GameUserUpdate {
                health: Some(100),
                credits: Some(0),
            })
            .filter(game_users::id.eq(game_user.id))
            .execute(c);

        delete(game_user_avatar_choices::table)
            .filter(game_user_avatar_choices::game_user_id.eq(game_user.id))
            .execute(c);

        if game
            .next_battle
            .map(|next| {
                Utc::now()
                    .signed_duration_since(DateTime::<Utc>::from_utc(next, Utc))
                    .num_seconds()
            })
            .map_or(false, |seconds| seconds > 5)
        {
            if let Ok(choices_left) = GameUserAvatarChoice::belonging_to(&game)
                .select(count_star())
                .first::<i64>(c)
            {
                if choices_left == 0 {
                    update(games::table)
                        .set(
                            games::next_battle
                                .eq(Utc::now().naive_utc() + chrono::Duration::seconds(5)),
                        )
                        .filter(games::id.eq(game.id))
                        .execute(c);
                }
            }
        }

        choosen_avatar
    })
    .await
    .unwrap();
}
