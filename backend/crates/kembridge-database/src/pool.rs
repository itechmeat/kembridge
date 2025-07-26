use sqlx::PgPool;

pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}