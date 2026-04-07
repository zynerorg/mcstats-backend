// @generated automatically by Diesel CLI.

diesel::table! {
    player_stats (player_uuid, stat_categories_id) {
        player_uuid -> Uuid,
        stat_categories_id -> Int4,
        value -> Numeric,
    }
}

diesel::table! {
    players (player_uuid) {
        player_uuid -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    stat_categories (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::joinable!(player_stats -> players (player_uuid));
diesel::joinable!(player_stats -> stat_categories (stat_categories_id));

diesel::allow_tables_to_appear_in_same_query!(player_stats, players, stat_categories,);
