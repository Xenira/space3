-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE OR REPLACE FUNCTION set_updated_at_date() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = now();
RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TABLE games (
	id SERIAL PRIMARY KEY,
	next_battle TIMESTAMP,
	current_round INT NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_games_updated_at BEFORE
UPDATE ON games FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	username VARCHAR(32) UNIQUE NOT NULL,
	password VARCHAR(255) NOT NULL,
	salt VARCHAR NOT NULL,
	currency INT NOT NULL DEFAULT 0,
	tutorial BOOLEAN NOT NULL DEFAULT 'f',
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_users_updated_at BEFORE
UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
CREATE TABLE lobbies (
	id SERIAL PRIMARY KEY,
	name VARCHAR(255) NOT NULL UNIQUE,
	passphrase VARCHAR(255),
	master_id INT NOT NULL,
	start_at TIMESTAMP,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT fk_master FOREIGN KEY(master_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_lobbies_updated_at BEFORE
UPDATE ON lobbies FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
CREATE TABLE lobby_users (
	id SERIAL PRIMARY KEY,
	lobby_id INT NOT NULL,
	user_id INT NOT NULL UNIQUE,
	username VARCHAR(32) UNIQUE NOT NULL,
	ready BOOLEAN NOT NULL DEFAULT 'f',
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT uq_lobby_user UNIQUE(lobby_id, user_id),
	CONSTRAINT fk_lobby FOREIGN KEY(lobby_id) REFERENCES lobbies(id) ON DELETE CASCADE,
	CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
	CONSTRAINT fk_username FOREIGN KEY(username) REFERENCES users(username) ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE TRIGGER update_lobby_users_updated_at BEFORE
UPDATE ON lobby_users FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
CREATE TABLE game_users (
	id SERIAL PRIMARY KEY,
	game_id INT NOT NULL,
	user_id INT NOT NULL UNIQUE,
	avatar_id INT,
	health INT NOT NULL,
	credits INT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT uq_game_user UNIQUE(game_id, user_id),
	CONSTRAINT fk_game FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
	CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_game_users_updated_at BEFORE
UPDATE ON game_users FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
CREATE TABLE game_user_avatar_choices (
	id SERIAL PRIMARY KEY,
	game_id INT NOT NULL,
	game_user_id INT NOT NULL,
	avatar_id INT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT uq_game_avatar UNIQUE(game_id, avatar_id),
	CONSTRAINT fk_game FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
	CONSTRAINT fk_game_user FOREIGN KEY(game_user_id) REFERENCES game_users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_game_user_avatar_choices_updated_at BEFORE
UPDATE ON game_user_avatar_choices FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
create TABLE game_user_characters (
	game_user_id INT PRIMARY KEY,
	character_id INT NOT NULL,
	position INT NOT NULL,
	upgraded BOOLEAN NOT NULL DEFAULT 'f',
	attack_bonus INT NOT NULL DEFAULT 0,
	defense_bonus INT NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	UNIQUE(game_user_id, position),
	CONSTRAINT fk_game_user FOREIGN KEY(game_user_id) REFERENCES game_users(id) ON DELETE CASCADE,
	CHECK (
		position >= 0
		AND position < 12
	) -- 7 for board + 5 for hand
);
CREATE TRIGGER update_game_user_characters_updated_at BEFORE
UPDATE ON game_user_characters FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();
create TABLE shops (
	id SERIAL PRIMARY KEY,
	game_user_id INT NOT NULL,
	character_ids INT[8] NOT NULL,
	locked BOOLEAN NOT NULL DEFAULT 'f',
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT fk_game_user FOREIGN KEY(game_user_id) REFERENCES game_users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_shops_updated_at BEFORE
UPDATE ON shops FOR EACH ROW EXECUTE PROCEDURE set_updated_at_date();