CREATE TABLE players (
    player_uuid TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE stat_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE player_stats (
    player_uuid TEXT NOT NULL REFERENCES players(player_uuid),
    stat_categories_id INTEGER NOT NULL REFERENCES stat_categories(id),
    stat_name TEXT NOT NULL,
    value INTEGER NOT NULL,

    PRIMARY KEY (player_uuid, stat_categories_id, stat_name)
);
