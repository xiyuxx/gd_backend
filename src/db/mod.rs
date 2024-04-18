use rocket_db_pools::{sqlx::PgPool,Database, Initializer,Connection};

#[derive(Database)]
#[database("graduationDesign")]
pub struct GdData(PgPool);

pub fn init_gd_data() -> Initializer<GdData> {
    GdData::init()
}

pub type GdDBC = Connection<GdData>;

pub type DbQueryResult<T> = Result<T, sqlx::Error>;
pub type SqlxError = sqlx::Error;