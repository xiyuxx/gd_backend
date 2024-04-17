use rocket_db_pools::{sqlx::PgPool,Database, Initializer,Connection};

#[derive(Database)]
#[database("gd_data")]
pub struct GdData(PgPool);

pub fn init_gd_data() -> Initializer<GdData> {
    GdData::init()
}

pub type GdDBC = Connection<GdData>;