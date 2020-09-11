use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
pub async fn get_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    info!("Connected to the database at url {}", &env::var("DATABASE_URL")?);
    Ok(pool)
}
