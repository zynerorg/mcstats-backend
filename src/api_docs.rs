use crate::entities::player::Model as Player;
use crate::entities::player_stats::Model as PlayerStats;
use crate::entities::stat_categorie::Model as StatCategorie;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Minecraft Stats API", version = "1.0"),
    components(schemas(Player, PlayerStats, StatCategorie))
)]
pub struct ApiDoc;
