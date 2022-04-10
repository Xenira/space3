table! {
    game_user_characters (game_user_id) {
        game_user_id -> Int4,
        character_id -> Int4,
        position -> Int4,
        upgraded -> Bool,
        attack_bonus -> Int4,
        defense_bonus -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    game_users (game_id, user_id) {
        id -> Int4,
        game_id -> Int4,
        user_id -> Int4,
        health -> Int4,
        credits -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    games (id) {
        id -> Int4,
        next_battle -> Nullable<Timestamp>,
        current_round -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        published -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        currency -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(game_users -> games (game_id));
joinable!(game_users -> users (user_id));

allow_tables_to_appear_in_same_query!(
    game_user_characters,
    game_users,
    games,
    users,
);
