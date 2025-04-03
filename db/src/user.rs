use common::{
    error::{AppError, Res},
    misc::UserVerificationOrigin,
};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    dtos::user::{AuthProviderCreateRequest, UserCreateRequest},
    models::user::{AuthCredentials, User},
};

pub async fn exists_user_by_email<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<bool> {
    sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists",
        email
    )
    .fetch_one(executor)
    .await
    .map(|row| row.exists.unwrap_or(false))
    .map_err(AppError::from)
}
pub async fn get_user_by_email<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}
pub async fn get_user_by_id<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    user_id: Uuid,
) -> Res<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(executor)
        .await
        .map_err(AppError::from)
}

pub async fn insert_user<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: UserCreateRequest,
) -> Res<User> {
    let verified = data.verification_origin == UserVerificationOrigin::OAuth;
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, first_name, last_name, company_name, verification_origin, verified, stripe_customer_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        data.email,
        data.first_name,
        data.last_name,
        data.company_name,
        data.verification_origin.to_string(),
        verified,
        data.stripe_customer_id
    )
    .fetch_one(executor)
    .await
    .map_err(AppError::from)
}

pub async fn insert_user_with_provider<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: AuthProviderCreateRequest,
) -> Res<()> {
    sqlx::query!(
        r#"
        INSERT INTO auth_providers (user_id, provider, provider_user_id)
        VALUES ($1, $2, $3)
        "#,
        data.user_id,
        data.provider,
        data.provider_user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn insert_user_with_credentials<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    data: AuthCredentials,
) -> Res<()> {
    sqlx::query!(
        r#"
        INSERT INTO auth_credentials (user_id, password_hash)
        VALUES ($1, $2)
        "#,
        data.user_id,
        data.password_hash
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn get_user_with_password_hash<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    email: String,
) -> Res<(User, AuthCredentials)> {
    sqlx::query!(
        r#"
        SELECT u.*, ac.password_hash
        FROM users u
        JOIN auth_credentials ac ON u.id = ac.user_id
        WHERE u.email = $1
        "#,
        email
    )
    .fetch_one(executor)
    .await
    .map(|record| {
        (
            User {
                id: record.id,
                email: record.email,
                first_name: record.first_name,
                last_name: record.last_name,
                company_name: record.company_name,
                created_at: record.created_at,
                updated_at: record.updated_at,
                verification_origin: record.verification_origin,
                verified: record.verified,
                stripe_customer_id: record.stripe_customer_id,
            },
            AuthCredentials {
                user_id: record.id,
                password_hash: record.password_hash,
            },
        )
    })
    .map_err(AppError::from)
}
