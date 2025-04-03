use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, password_hash::PasswordHasher};
use common::env_config::Config;
use common::error::Res;
use common::misc::UserVerificationOrigin;
use common::stripe;
use db::dtos::user::{AuthProviderCreateRequest, UserCreateRequest};
use db::models::user::{AuthCredentials, User};
use crate::dtos::auth::{OAuthUserData, RegisterRequest};
use crate::misc::oauth::OAuthProvider;

use sqlx::PgPool;
use uuid::Uuid;

pub async fn exists_user_by_email(pool: &PgPool, email: String) -> Res<bool> {
    db::user::exists_user_by_email(pool, email).await
}
pub async fn get_user_by_email(pool: &PgPool, email: String) -> Res<User> {
    db::user::get_user_by_email(pool, email).await
}
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Res<User> {
    db::user::get_user_by_id(pool, user_id).await
}

/// Inserts user record and OAuth data to the database.
/// Used when signing in using OAuth provider.
pub async fn create_user_with_oauth(
    pool: &PgPool,
    user_data: &OAuthUserData,
    provider: &OAuthProvider,
    config: &Config,
) -> Res<User> {
    let mut tx = pool.begin().await?;

    // create Stripe customer
    let client = stripe::create_client(&config.stripe_secret_key);
    let name = format!("{} {}", user_data.first_name, user_data.last_name);
    let stripe_customer =
        stripe::create_customer(&client, &user_data.email, &name).await?;

    // insert user
    let user = db::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: user_data.email.clone(),
            first_name: user_data.first_name.clone(),
            last_name: user_data.last_name.clone(),
            company_name: None,
            verification_origin: UserVerificationOrigin::OAuth,
            stripe_customer_id: Some(stripe_customer.id.to_string()),
        },
    )
    .await?;

    // insert provider's user data
    db::user::insert_user_with_provider(
        &mut *tx,
        AuthProviderCreateRequest {
            user_id: user.id,
            provider: provider.as_str().to_string(),
            provider_user_id: user_data.provider_user_id.clone(),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(user)
}

/// Inserts user record and credentials to the database.
/// User when signing in using credentials.
pub async fn create_user_with_credentials(
    pool: &PgPool,
    req: &RegisterRequest,
    config: &Config,
) -> Res<User> {
    let mut tx = pool.begin().await?;

    // create Stripe customer
    let client = stripe::create_client(&config.stripe_secret_key);
    let name = format!("{} {}", req.first_name, req.last_name);
    let stripe_customer =
        stripe::create_customer(&client, &req.email, &name).await?;

    // insert user
    let user = db::user::insert_user(
        &mut *tx,
        UserCreateRequest {
            email: req.email.clone(),
            first_name: req.first_name.clone(),
            last_name: req.last_name.clone(),
            company_name: req.company_name.clone(),
            verification_origin: UserVerificationOrigin::Email,
            stripe_customer_id: Some(stripe_customer.id.to_string()),
        },
    )
    .await?;

    // hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // insert credentials
    db::user::insert_user_with_credentials(
        &mut *tx,
        AuthCredentials {
            user_id: user.id,
            password_hash,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(user)
}
