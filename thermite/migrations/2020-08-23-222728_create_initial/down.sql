BEGIN;

DROP TABLE IF EXISTS thermite_games_bans CASCADE;

DROP TABLE IF EXISTS thermite_games_members CASCADE;

DROP TABLE IF EXISTS thermite_games_apikeys CASCADE;

DROP TABLE IF EXISTS thermite_games CASCADE;

DROP TABLE IF EXISTS thermite_users_storage CASCADE;

DROP TABLE IF EXISTS thermite_users_sessions CASCADE;

DROP TABLE IF EXISTS thermite_users_emails CASCADE;

DROP TABLE IF EXISTS thermite_users_bans CASCADE;

DROP TABLE IF EXISTS thermite_users_passwords CASCADE;

DROP TABLE IF EXISTS thermite_users_profiles CASCADE;

DROP TABLE IF EXISTS thermite_users_email CASCADE;

DROP TABLE IF EXISTS thermite_users CASCADE;

COMMIT;