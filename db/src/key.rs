use common::error::{AppError, Res};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    dtos::key::KeyCreateRequest,
    models::key::ApiKey,
};

pub async fn get_key_by_id<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    key_id: &Uuid,
) -> Res<ApiKey> {
    sqlx::query_as!(ApiKey, "SELECT * FROM api_keys WHERE id = $1", key_id)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}

pub async fn get_keys_by_user_id<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: &Uuid,
) -> Res<Vec<ApiKey>> {
    sqlx::query_as!(ApiKey, "SELECT * FROM api_keys WHERE user_id = $1", user_id)
        .fetch_all(executor)
        .await
        .map_err(AppError::from)
}

pub async fn get_active_key_by_key_encrypted<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    key: &str,
) -> Res<ApiKey> {
    sqlx::query_as!(ApiKey, "SELECT * FROM api_keys WHERE key_encrypted = $1 AND status = 'active'", key)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}

pub async fn insert_key<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: KeyCreateRequest,
) -> Res<ApiKey> {
    sqlx::query_as!(
        ApiKey,
        r#"
        INSERT INTO api_keys (user_id, key_encrypted, name, status, permissions)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        data.user_id,
        data.key_encrypted,
        data.name,
        "active",
        data.permissions
    )
    .fetch_one(executor)
    .await
    .map_err(AppError::from)
}

pub async fn update_key_status<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    key_id: Uuid,
    status: &str
) -> Res<ApiKey> {
    let updated_key = sqlx::query_as!(ApiKey,
        "UPDATE api_keys SET status = $1 WHERE id = $2 RETURNING *",
        status,
        key_id,
    )
    .fetch_one(executor)
    .await
    .map_err(AppError::from)?;

    Ok(updated_key)
}
