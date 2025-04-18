use common::{
    error::{AppError, Res},
    jwt::JwtClaims,
    key::KeyClaims,
    misc::hash_str,
};
use db::{dtos::key::KeyCreateRequest, models::key::ApiKey};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::key::{ApiKeyListItem, CreateKeyRequest, CreateKeyResponse};

/// Retrieves a list of API keys for a given user ID.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `user_id` - The ID of the user for whom to retrieve API keys.
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyListItem` objects or an `AppError` if an error occurs.
pub async fn get_keys(pool: &PgPool, user_id: Uuid) -> Res<Vec<ApiKeyListItem>> {
    let api_keys = db::key::get_keys_by_user_id(pool, &user_id).await?;

    let api_key_list_items = api_keys
        .into_iter()
        .map(|key| ApiKeyListItem {
            id: key.id,
            user_id: key.user_id,
            key_hashed: key.key_encrypted,
            name: key.name,
            status: key.status,
            created_at: key.created_at,
            permissions: key.permissions,
        })
        .collect();

    Ok(api_key_list_items)
}

/// Creates a new API key for a user.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `claims` - The JWT claims of the user creating the key.
/// * `stripe_secret` - The Stripe secret key.
/// * `req` - The request containing the information for creating the key.
///
/// # Returns
///
/// A `Result` containing a `CreateKeyResponse` object or an `AppError` if an error occurs.
pub async fn create_key(
    pool: &PgPool,
    claims: JwtClaims,
    stripe_secret: &str,
    req: CreateKeyRequest,
) -> Res<CreateKeyResponse> {
    let user_id = claims.user_id;
    let customer_id = &claims.stripe_customer_id;

    // get plan id
    let client = common::stripe::create_client(stripe_secret);
    let plan = api_subs::services::sub::get_user_subscription(&client, customer_id).await?;
    let plan_id = if let Some(plan) = plan {
        plan.id
    } else {
        return Err(AppError::BadRequest(
            "Tried to create an API key for user with no active subscription plan".to_string(),
        ));
    };

    // generate a secret token
    let secret = generate_secret();

    // insert hashed secret
    let db_key = db::key::insert_key(
        pool,
        KeyCreateRequest {
            user_id,
            key_encrypted: hash_str(secret.as_str()),
            name: req.name,
            permissions: req.permissions,
        },
    )
    .await?;

    // construct claims
    let key_claims = KeyClaims {
        user_id,
        plan_id,
        secret,
        key_id: db_key.id,
    };

    // serialize claims into key
    let key = key_claims.to_key();

    Ok(CreateKeyResponse {
        id: db_key.id,
        key,
        user_id,
        name: db_key.name,
        status: db_key.status,
        created_at: db_key.created_at,
        permissions: db_key.permissions,
    })
}

/// Updates the status of an API key.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `key_id` - The ID of the key to update.
/// * `status` - The new status of the key.
///
/// # Returns
///
/// A `Result` containing the updated `ApiKey` object or an `AppError` if an error occurs.
pub async fn update_key_status(pool: &PgPool, key_id: Uuid, status: &str) -> Res<ApiKey> {
    db::key::update_key_status(pool, key_id, status).await
}

/// Generates a secret key.
///
/// # Returns
///
/// A randomly generated UUID as a string.
fn generate_secret() -> String {
    Uuid::new_v4().to_string()
}
