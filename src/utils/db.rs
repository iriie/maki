use sqlx::postgres::PgPool;
use std::env;
pub async fn get_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPool::builder()
        .max_size(20)
        .build(&env::var("DATABASE_URL")?)
        .await?;
    Ok(pool)
}
