BEGIN;

CREATE TABLE IF NOT EXISTS thermite_users (
    id SERIAL NOT NULL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    joined timestamp with time zone NOT NULL,
    password_hash text NOT NULL,
    active boolean NOT NULL DEFAULT true,
    is_superuser boolean NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS thermite_users_email (
    id SERIAL NOT NULL PRIMARY KEY,
    address VARCHAR(255) UNIQUE NOT NULL,
    added timestamp with time zone NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,
    verified_at timestamp with time zone NULL
);

CREATE TABLE IF NOT EXISTS thermite_users_profiles (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL UNIQUE REFERENCES thermite_users(id) ON DELETE CASCADE,
    display_name VARCHAR(255),
    email INT NULL REFERENCES thermite_users_email(id) ON DELETE RESTRICT,
    lang_tag VARCHAR(18) NOT NULL DEFAULT 'en',
    timezone VARCHAR(255) NOT NULL DEFAULT 'UTC'
);

CREATE TABLE IF NOT EXISTS thermite_users_passwords (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES thermite_users(id) ON DELETE CASCADE,
    password_hash text NOT NULL,
    added timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS thermite_users_bans (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL UNIQUE REFERENCES thermite_users(id) ON DELETE CASCADE,
    banned_on timestamp with time zone NOT NULL,
    banned_until timestamp with time zone NOT NULL,
    banned_by INT NULL REFERENCES thermite_users(id) ON DELETE SET NULL,
    ban_reason text NOT NULL
);

CREATE TABLE IF NOT EXISTS thermite_users_emails (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES thermite_users(id) ON DELETE CASCADE,
    email_id INT NOT NULL REFERENCES thermite_users_email(id) ON DELETE RESTRICT,
    added timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS thermite_users_sessions (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES thermite_users(id) ON DELETE CASCADE,
    created timestamp with time zone NOT NULL,
    valid_until timestamp with time zone NOT NULL,
    session_key VARCHAR(16) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS thermite_users_storage (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES thermite_users(id) ON DELETE CASCADE,
    category VARCHAR(255) NOT NULL,
    storage_name VARCHAR(255) NOT NULL,
    created timestamp with time zone NOT NULL,
    modified timestamp with time zone NOT NULL,
    json_data JSON,
    UNIQUE(user_id, category, storage_name)
);

CREATE TABLE IF NOT EXISTS thermite_games (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES thermite_users(id) ON DELETE RESTRICT,
    gamename VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    created timestamp with time zone NOT NULL,
    is_public bool NOT NULL DEFAULT false,
    UNIQUE(user_id, gamename)
);

CREATE TABLE IF NOT EXISTS thermite_games_apikeys (
    id SERIAL NOT NULL PRIMARY KEY,
    game_id INT NOT NULL REFERENCES thermite_games(id) ON DELETE CASCADE,
    game_key VARCHAR(16) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS thermite_games_members (
    id SERIAL NOT NULL PRIMARY KEY,
    game_id INT NOT NULL REFERENCES thermite_games(id) ON DELETE CASCADE,
    user_id INT NOT NULL,
    joined timestamp with time zone NOT NULL,
    member_key VARCHAR(16) NOT NULL,
    UNIQUE(game_id, user_id),
    UNIQUE(game_id, member_key)
);

CREATE TABLE IF NOT EXISTS thermite_games_bans (
    id SERIAL NOT NULL PRIMARY KEY,
    game_id INT NOT NULL REFERENCES thermite_games(id) ON DELETE CASCADE,
    user_id INT NOT NULL,
    banned_on timestamp with time zone NOT NULL,
    banned_until timestamp with time zone NOT NULL,
    banned_by INT NULL,
    ban_reason text NOT NULL
);

COMMIT;
