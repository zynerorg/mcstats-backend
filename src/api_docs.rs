use crate::entities::player::Model as Player;
use crate::entities::player_stats::Model as PlayerStats;
use crate::entities::stat_categories::Model as StatCategory;
use crate::server::categories::__path_categories;
use crate::server::categories::__path_category;
use crate::server::players::__path_player;
use crate::server::players::__path_players;
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
    paths(category, categories, players, player),
    components(schemas(Player, PlayerStats, StatCategory))
)]
pub struct ApiDoc;
