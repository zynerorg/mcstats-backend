use crate::entities::player::Model as Player;
use crate::entities::player_stats::Model as PlayerStats;
use crate::entities::stat_categories::Model as StatCategorie;
use crate::server::__path_categorie;
use crate::server::__path_category;
use crate::server::__path_player;
use crate::server::__path_players;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Minecraft Stats API",
        version = "1.0",
        license(
            name = "GPL-3.0-or-later",
            url = "https://www.gnu.org/licenses/gpl-3.0.html"
        )
    ),
    paths(category, categorie, players, player),
    components(schemas(Player, PlayerStats, StatCategorie))
)]
pub struct ApiDoc;
