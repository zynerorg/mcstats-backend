// @generated automatically by Diesel CLI.

diesel::table! {
    player_stats (player_uuid, stat_categories_id, stat_name) {
        player_uuid -> Text,
        stat_categories_id -> Integer,
        stat_name -> Text,
        value -> Integer,
    }
}

diesel::table! {
    players (player_uuid) {
        player_uuid -> Text,
        name -> Text,
    }
}

diesel::table! {
    stat_categories (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(player_stats -> players (player_uuid));
diesel::joinable!(player_stats -> stat_categories (stat_categories_id));

diesel::allow_tables_to_appear_in_same_query!(player_stats, players, stat_categories,);
