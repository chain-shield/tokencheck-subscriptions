use common::env_config::Config;

pub async fn setup_redis(config: &Config) -> deadpool_redis::Pool {
    let cfg = deadpool_redis::Config::from_url(&config.redis_url);
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create pool of Redis connections")
}
