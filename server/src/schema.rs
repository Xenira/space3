// @generated automatically by Diesel CLI.

diesel::table! {
    game_user_avatar_choices (id) {
        id -> Int4,
        game_id -> Int4,
        game_user_id -> Int4,
        avatar_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    game_user_characters (id) {
        id -> Int4,
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

diesel::table! {
    game_users (id) {
        id -> Int4,
        game_id -> Int4,
        user_id -> Int4,
        avatar_id -> Nullable<Int4>,
        experience -> Int4,
        health -> Int4,
        credits -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    games (id) {
        id -> Int4,
        next_battle -> Nullable<Timestamp>,
        current_round -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    lobbies (id) {
        id -> Int4,
        name -> Varchar,
        passphrase -> Nullable<Varchar>,
        master_id -> Int4,
        start_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    lobby_users (id) {
        id -> Int4,
        lobby_id -> Int4,
        user_id -> Int4,
        username -> Varchar,
        ready -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    shops (id) {
        id -> Int4,
        game_id -> Int4,
        game_user_id -> Int4,
        character_ids -> Array<Nullable<Int4>>,
        locked -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        currency -> Int4,
        tutorial -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(lobbies -> users (master_id));
diesel::joinable!(lobby_users -> lobbies (lobby_id));

diesel::allow_tables_to_appear_in_same_query!(
    game_user_avatar_choices,
    game_user_characters,
    game_users,
    games,
    lobbies,
    lobby_users,
    shops,
    users,
);
