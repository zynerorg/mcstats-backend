use crate::entities::player::Model as Player;
use crate::entities::player_stats::Model as PlayerStats;
use crate::entities::stat_categorie::Model as StatCategorie;
use crate::server::__path_categorie;
use crate::server::__path_categories;
use crate::server::__path_player;
use crate::server::__path_players;
use crate::server::{categorie, categories, player, players};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Minecraft Stats API", version = "1.0"),
    paths(categories, categorie, players, player),
    components(schemas(Player, PlayerStats, StatCategorie))
)]
pub struct ApiDoc;
