use crate::models::{Player, PlayerStats, StatCategorie};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Minecraft Stats API", version = "1.0"),
    components(schemas(Player, PlayerStats, StatCategorie))
)]
pub struct ApiDoc;
