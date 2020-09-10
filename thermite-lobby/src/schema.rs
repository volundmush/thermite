table! {
    board (id) {
        id -> Integer,
        board_id -> Integer,
        game_id -> Nullable<Integer>,
        name -> Text,
        display_name -> Text,
        mandatory -> Bool,
        restricted -> Bool,
        next_id -> Integer,
    }
}

table! {
    channel (id) {
        id -> Integer,
        game_id -> Nullable<Integer>,
        name -> Text,
        display_name -> Nullable<Text>,
        restricted -> Bool,
    }
}

table! {
    channelsub (id) {
        id -> Integer,
        channel_id -> Integer,
        user_id -> Integer,
        command -> Text,
        status -> Bool,
    }
}

table! {
    game (id) {
        id -> Integer,
        user_id -> Integer,
        abbr -> Text,
        display_name -> Text,
        created -> Timestamp,
        active -> Bool,
        is_public -> Bool,
        game_key -> Text,
        banned_until -> Nullable<Timestamp>,
        banned_by -> Nullable<Integer>,
        ban_reason -> Nullable<Text>,
    }
}

table! {
    gamemember (id) {
        id -> Integer,
        game_id -> Integer,
        user_id -> Integer,
        joined -> Timestamp,
        member_key -> Text,
        active -> Bool,
        is_superuser -> Bool,
        is_admin -> Bool,
        banned_until -> Nullable<Timestamp>,
        banned_by -> Nullable<Integer>,
        ban_reason -> Nullable<Text>,
    }
}

table! {
    memberstorage (id) {
        id -> Integer,
        member_id -> Integer,
        category -> Text,
        storage_name -> Text,
        created -> Timestamp,
        modified -> Timestamp,
        json_data -> Nullable<Text>,
    }
}

table! {
    post (id) {
        id -> Integer,
        board_id -> Integer,
        post_num -> Integer,
        user_id -> Integer,
        subject -> Text,
        body -> Text,
        date_created -> Timestamp,
        date_modified -> Timestamp,
    }
}

table! {
    postread (id) {
        id -> Integer,
        post_id -> Integer,
        user_id -> Integer,
        date_checked -> Timestamp,
    }
}

table! {
    user (id) {
        id -> Integer,
        username -> Text,
        date_joined -> Timestamp,
        password_hash -> Text,
        email -> Nullable<Text>,
        email_verified -> Bool,
        email_verified_on -> Nullable<Timestamp>,
        active -> Bool,
        is_superuser -> Bool,
        is_admin -> Bool,
        timezone -> Text,
        banned_until -> Nullable<Timestamp>,
        banned_by -> Nullable<Integer>,
        ban_reason -> Nullable<Text>,
    }
}

table! {
    usersession (id) {
        id -> Integer,
        user_id -> Integer,
        created -> Timestamp,
        valid_until -> Timestamp,
        session_key -> Text,
    }
}

table! {
    userstorage (id) {
        id -> Integer,
        user_id -> Integer,
        category -> Text,
        storage_name -> Text,
        created -> Timestamp,
        modified -> Timestamp,
        json_data -> Nullable<Text>,
    }
}

joinable!(channel -> game (game_id));
joinable!(channelsub -> channel (channel_id));
joinable!(channelsub -> user (user_id));
joinable!(game -> user (user_id));
joinable!(gamemember -> game (game_id));
joinable!(memberstorage -> gamemember (member_id));
joinable!(post -> board (board_id));
joinable!(post -> user (user_id));
joinable!(postread -> post (post_id));
joinable!(postread -> user (user_id));
joinable!(usersession -> user (user_id));
joinable!(userstorage -> user (user_id));

allow_tables_to_appear_in_same_query!(
    board,
    channel,
    channelsub,
    game,
    gamemember,
    memberstorage,
    post,
    postread,
    user,
    usersession,
    userstorage,
);
