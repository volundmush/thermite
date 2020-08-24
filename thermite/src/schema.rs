table! {
    accountcomponent (id) {
        id -> Uuid,
        password -> Text,
        email -> Varchar,
        superuser -> Bool,
    }
}

table! {
    aclentry (id) {
        id -> Int8,
        resource -> Uuid,
        target -> Uuid,
        mode -> Varchar,
        deny -> Bool,
    }
}

table! {
    acllink (id) {
        id -> Int8,
        acl_id -> Int8,
        perm_id -> Int4,
    }
}

table! {
    aclpermission (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    attributes (id) {
        id -> Int8,
        owner -> Uuid,
        category -> Varchar,
        name -> Varchar,
        data -> Json,
    }
}

table! {
    childcomponent (id) {
        id -> Uuid,
        parent -> Uuid,
        child_type_id -> Int4,
        child_key -> Varchar,
    }
}

table! {
    childtype (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    entities (id) {
        id -> Uuid,
        type_id -> Int4,
        python_path -> Varchar,
    }
}

table! {
    entitylocationcomponent (id) {
        id -> Uuid,
        location -> Uuid,
    }
}

table! {
    equipslotcomponent (id) {
        id -> Uuid,
        slot_key -> Varchar,
        slot_layer -> Int4,
    }
}

table! {
    fixturecomponent (id) {
        id -> Uuid,
        fixture_space_id -> Int4,
        fixture_key -> Varchar,
    }
}

table! {
    fixturespace (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    floatpositioncomponent (id) {
        id -> Uuid,
        x -> Float8,
        y -> Float8,
        z -> Float8,
    }
}

table! {
    intpositioncomponent (id) {
        id -> Uuid,
        x -> Int4,
        y -> Int4,
        z -> Int4,
    }
}

table! {
    namecomponent (id) {
        id -> Uuid,
        namespace_id -> Int4,
        name -> Varchar,
        color_name -> Varchar,
    }
}

table! {
    namespace (id) {
        id -> Int4,
        name -> Varchar,
        searchable -> Bool,
    }
}

table! {
    playercharactercomponent (id) {
        id -> Uuid,
        account_id -> Uuid,
    }
}

table! {
    pluginname (id) {
        id -> Int4,
        name -> Varchar,
    }
}

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
        game_key -> Varchar,
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
        member_key -> Varchar,
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
        session_key -> Varchar,
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

table! {
    types (id) {
        id -> Int4,
        name -> Varchar,
    }
}

joinable!(accountcomponent -> entities (id));
joinable!(acllink -> aclentry (acl_id));
joinable!(acllink -> aclpermission (perm_id));
joinable!(attributes -> entities (owner));
joinable!(childcomponent -> childtype (child_type_id));
joinable!(entities -> types (type_id));
joinable!(equipslotcomponent -> entities (id));
joinable!(fixturecomponent -> entities (id));
joinable!(fixturecomponent -> fixturespace (fixture_space_id));
joinable!(floatpositioncomponent -> entities (id));
joinable!(intpositioncomponent -> entities (id));
joinable!(namecomponent -> entities (id));
joinable!(namecomponent -> namespace (namespace_id));
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
    accountcomponent,
    aclentry,
    acllink,
    aclpermission,
    attributes,
    childcomponent,
    childtype,
    entities,
    entitylocationcomponent,
    equipslotcomponent,
    fixturecomponent,
    fixturespace,
    floatpositioncomponent,
    intpositioncomponent,
    namecomponent,
    namespace,
    playercharactercomponent,
    pluginname,
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
    types,
);
