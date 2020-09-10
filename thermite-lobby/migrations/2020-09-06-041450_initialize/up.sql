CREATE TABLE IF NOT EXISTS user (
    id INTEGER NOT NULL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    date_joined TIMESTAMP NOT NULL,
    password_hash text NOT NULL,
    email VARCHAR(255) UNIQUE COLLATE NOCASE,
    email_verified boolean NOT NULL DEFAULT false,
    email_verified_on TIMESTAMP,
    active boolean NOT NULL DEFAULT true,
    is_superuser boolean NOT NULL DEFAULT false,
    is_admin boolean NOT NULL DEFAULT false,
    timezone VARCHAR(255) NOT NULL DEFAULT 'UTC',
    banned_until TIMESTAMP,
    banned_by INTEGER,
    ban_reason TEXT
);

CREATE TABLE IF NOT EXISTS usersession (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INT NOT NULL,
    created TIMESTAMP NOT NULL,
    valid_until TIMESTAMP NOT NULL,
    session_key VARCHAR(16) NOT NULL UNIQUE,
    FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS userstorage (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    category VARCHAR(255) NOT NULL,
    storage_name VARCHAR(255) NOT NULL,
    created TIMESTAMP NOT NULL,
    modified TIMESTAMP NOT NULL,
    json_data TEXT,
    FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE(user_id, category, storage_name)
);

CREATE TABLE IF NOT EXISTS game (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    abbr VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,
    display_name VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,
    created TIMESTAMP NOT NULL,
    active BOOLEAN NOT NULL DEFAULT True,
    is_public BOOLEAN NOT NULL DEFAULT false,
    game_key VARCHAR(16) NOT NULL UNIQUE,
    banned_until TIMESTAMP,
    banned_by INTEGER,
    ban_reason TEXT,
    FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    UNIQUE(user_id, abbr)
);

CREATE TABLE IF NOT EXISTS gamemember (
    id INTEGER NOT NULL PRIMARY KEY,
    game_id INT NOT NULL,
    user_id INT NOT NULL,
    joined TIMESTAMP NOT NULL,
    member_key VARCHAR(16) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT True,
    is_superuser BOOLEAN NOT NULL DEFAULT false,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    banned_until TIMESTAMP,
    banned_by INTEGER,
    ban_reason TEXT,
    FOREIGN KEY(game_id) REFERENCES game(id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE(game_id, user_id),
    UNIQUE(game_id, member_key)
);

CREATE TABLE IF NOT EXISTS memberstorage (
    id INTEGER NOT NULL PRIMARY KEY,
    member_id INT NOT NULL,
    category VARCHAR(255) NOT NULL,
    storage_name VARCHAR(255) NOT NULL,
    created TIMESTAMP NOT NULL,
    modified TIMESTAMP NOT NULL,
    json_data TEXT,
    FOREIGN KEY(member_id) REFERENCES gamemember(id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE(member_id, category, storage_name)
);

CREATE TABLE IF NOT EXISTS board (
    id INTEGER NOT NULL PRIMARY KEY,
    board_id INTEGER NOT NULL,
    game_id INTEGER,
    name VARCHAR(255) NOT NULL COLLATE NOCASE,
    display_name VARCHAR(255) NOT NULL,
    mandatory BOOLEAN NOT NULL DEFAULT false,
    restricted TINYINT NOT NULL DEFAULT 0,
    next_id INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS post (
    id INTEGER NOT NULL PRIMARY KEY,
    board_id INTEGER NOT NULL,
    post_num INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    date_created TIMESTAMP NOT NULL,
    date_modified TIMESTAMP NOT NULL,
    FOREIGN KEY(board_id) REFERENCES board(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE(board_id, post_num)
);

CREATE TABLE IF NOT EXISTS postread (
    id INTEGER NOT NULL PRIMARY KEY,
    post_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    date_checked TIMESTAMP NOT NULL,
    FOREIGN KEY(post_id) REFERENCES post(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE(user_id, post_id)
);

CREATE TABLE IF NOT EXISTS channel (
    id INTEGER NOT NULL PRIMARY KEY,
    game_id INTEGER,
    name VARCHAR(255) NOT NULL COLLATE NOCASE,
    display_name VARCHAR(255),
    restricted TINYINT NOT NULL DEFAULT 0,
    FOREIGN KEY(game_id) REFERENCES game(id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS channelsub (
    id INTEGER NOT NULL PRIMARY KEY,
    channel_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    command VARCHAR(255) NOT NULL,
    status TINYINT NOT NULL,
    FOREIGN KEY(channel_id) REFERENCES channel(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(user_id, command),
    UNIQUE(user_id, channel_id)
);