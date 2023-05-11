-- Your SQL goes here
CREATE TABLE games (
	id SERIAL PRIMARY KEY,
	next_battle TIMESTAMP,
	current_round INT NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	published BOOLEAN NOT NULL DEFAULT 'f'
);
CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	username VARCHAR(32) UNIQUE NOT NULL,
	password VARCHAR(255) NOT NULL,
	salt VARCHAR NOT NULL,
	currency INT NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE TABLE game_users (
	id SERIAL UNIQUE,
	game_id INT,
	user_id INT,
	health INT NOT NULL,
	credits INT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	PRIMARY KEY(game_id, user_id),
	CONSTRAINT fk_game FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
	CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
create TABLE game_user_characters (
	game_user_id INT PRIMARY KEY,
	character_id INT NOT NULL,
	position INT NOT NULL,
	upgraded BOOLEAN NOT NULL DEFAULT 'f',
	attack_bonus INT NOT NULL DEFAULT 0,
	defense_bonus INT NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	UNIQUE(game_user_id, position),
	CONSTRAINT fk_game_user FOREIGN KEY(game_user_id) REFERENCES game_users(id) ON DELETE CASCADE,
	CHECK (
		position >= 0
		AND position < 12
	) -- 7 for board + 5 for hand
);