use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Player {
    pub player_uuid: Uuid,
    pub name: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::players)]
pub struct NewPlayer {
    pub player_uuid: Uuid,
    pub name: String,
}
