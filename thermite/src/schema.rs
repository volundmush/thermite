table! {
    thermite_games (id) {
        id -> Int4,
        user_id -> Int4,
        gamename -> Varchar,
        display_name -> Varchar,
        created -> Timestamptz,
        is_public -> Bool,
    }
}

table! {
    thermite_games_apikeys (id) {
        id -> Int4,
        game_id -> Int4,
    }
}

table! {
    thermite_games_bans (id) {
        id -> Int4,
        game_id -> Int4,
        user_id -> Int4,
        banned_on -> Timestamptz,
        banned_until -> Timestamptz,
        banned_by -> Nullable<Int4>,
        ban_reason -> Text,
    }
}

table! {
    thermite_games_members (id) {
        id -> Int4,
        game_id -> Int4,
        user_id -> Int4,
        joined -> Timestamptz,
    }
}

table! {
    thermite_users (id) {
        id -> Int4,
        username -> Varchar,
        joined -> Timestamptz,
        password_hash -> Nullable<Text>,
        active -> Bool,
        is_superuser -> Bool,
    }
}

table! {
    thermite_users_bans (id) {
        id -> Int4,
        user_id -> Int4,
        banned_on -> Timestamptz,
        banned_until -> Timestamptz,
        banned_by -> Nullable<Int4>,
        ban_reason -> Text,
    }
}

table! {
    thermite_users_email (id) {
        id -> Int4,
        address -> Varchar,
        added -> Timestamptz,
        verified -> Bool,
        verified_at -> Nullable<Timestamptz>,
    }
}

table! {
    thermite_users_emails (id) {
        id -> Int4,
        user_id -> Int4,
        email_id -> Int4,
        added -> Timestamptz,
    }
}

table! {
    thermite_users_passwords (id) {
        id -> Int4,
        user_id -> Int4,
        password_hash -> Text,
        added -> Timestamptz,
    }
}

table! {
    thermite_users_profiles (id) {
        id -> Int4,
        user_id -> Int4,
        display_name -> Nullable<Varchar>,
        email -> Nullable<Int4>,
        lang_tag -> Varchar,
        timezone -> Varchar,
    }
}

table! {
    thermite_users_sessions (id) {
        id -> Int4,
        user_id -> Int4,
        created -> Timestamptz,
        valid_until -> Timestamptz,
    }
}

table! {
    thermite_users_storage (id) {
        id -> Int4,
        user_id -> Int4,
        category -> Varchar,
        storage_name -> Varchar,
        created -> Timestamptz,
        modified -> Timestamptz,
        json_data -> Nullable<Json>,
    }
}

joinable!(thermite_games -> thermite_users (user_id));
joinable!(thermite_games_apikeys -> thermite_games (game_id));
joinable!(thermite_games_bans -> thermite_games (game_id));
joinable!(thermite_games_members -> thermite_games (game_id));
joinable!(thermite_users_emails -> thermite_users (user_id));
joinable!(thermite_users_emails -> thermite_users_email (email_id));
joinable!(thermite_users_passwords -> thermite_users (user_id));
joinable!(thermite_users_profiles -> thermite_users (user_id));
joinable!(thermite_users_profiles -> thermite_users_email (email));
joinable!(thermite_users_sessions -> thermite_users (user_id));
joinable!(thermite_users_storage -> thermite_users (user_id));

allow_tables_to_appear_in_same_query!(
    thermite_games,
    thermite_games_apikeys,
    thermite_games_bans,
    thermite_games_members,
    thermite_users,
    thermite_users_bans,
    thermite_users_email,
    thermite_users_emails,
    thermite_users_passwords,
    thermite_users_profiles,
    thermite_users_sessions,
    thermite_users_storage,
);
