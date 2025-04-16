use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{str::FromStr, sync::Arc};

pub mod log;
pub mod user;
pub mod key;

pub mod models {
    pub mod key;
    pub mod log;
    pub mod user;
}

pub mod dtos {
    pub mod user;
    pub mod key;
    pub mod usage;
    pub mod log;
}

pub async fn setup(
    database_url: &str,
    require_ssl: bool,
) -> Result<Arc<PgPool>, Box<dyn std::error::Error>> {
    let url = url::Url::parse(database_url)?;
    let db_name = url.path().trim_start_matches('/');
    let username = url.username();
    let password = url.password().unwrap_or("");
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);

    let admin_url = format!(
        "postgresql://{}:{}@{}:{}/postgres",
        username, password, host, port
    );

    let mut admin_options = PgConnectOptions::from_str(&admin_url)?;
    if require_ssl {
        admin_options = admin_options.ssl_mode(PgSslMode::Require);
    }

    let admin_pool = PgPool::connect_with(admin_options).await?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&admin_pool)
            .await?;

    if !exists {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await?;
    }

    admin_pool.close().await;

    let mut options = PgConnectOptions::from_str(database_url)?;
    if require_ssl {
        options = options.ssl_mode(PgSslMode::Require);
    }
    let pool = PgPool::connect_with(options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Arc::new(pool))
}
