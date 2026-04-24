use crate::entities::items::Model as Item;
use crate::entities::player_stats::Model as PlayerStats;
use crate::entities::players::Model as Player;
use crate::entities::stat_categories::Model as StatCategory;
use crate::server::categories;
use crate::server::items;
use crate::server::players;
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
    paths(
        categories::categories,
        categories::category,
        players::players,
        players::player,
        items::items,
        items::item,
        items::stats,
    ),
    components(schemas(Player, PlayerStats, StatCategory, Item))
)]
pub struct ApiDoc;