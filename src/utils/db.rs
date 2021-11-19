use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
pub async fn get_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    let url = &env::var("DATABASE_URL")?;
    let mut split = url.split("@");
    split.next();
    let cleaned_url = split.next().unwrap().split("/").next().unwrap();
    info!("Connected to the database at {}", &cleaned_url);
    Ok(pool)
}