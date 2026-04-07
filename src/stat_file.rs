use serde::Deserialize;

pub type Categorie = Vec<(String, i32)>;

#[derive(Deserialize, Debug)]
pub struct StatsFile {
    dropped: Categorie,
    used: Categorie,
    mined: Categorie,
    picked_up: Categorie,
    killed: Categorie,
    crafted: Categorie,
    custom: Categorie,
    broken: Categorie,
}
