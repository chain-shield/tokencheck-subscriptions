use std::sync::Arc;

use actix_web::{
    Responder, get, post,
    web::{self},
};
use common::{env_config::Config, error::Res, http::Success, jwt::JwtClaims};
use sqlx::PgPool;

use crate::{
    dtos::key::{CreateKeyRequest, RevokeKeyRequest},
    service,
};

#[get("/keys")]
pub async fn get_keys(
    claims: web::ReqData<JwtClaims>,
    pool: web::Data<Arc<PgPool>>,
) -> Res<impl Responder> {
    let user_id = claims.user_id;
    let keys = service::key::get_keys(&pool, user_id).await?;
    Success::ok(keys)
}

#[post("/generate")]
pub async fn post_generate_key(
    config: web::Data<Arc<Config>>,
    claims: web::ReqData<JwtClaims>,
    pool: web::Data<Arc<PgPool>>,
    req: web::Json<CreateKeyRequest>,
) -> Res<impl Responder> {
    let key = service::key::create_key(
        &pool,
        claims.into_inner(),
        &config.stripe_secret_key,
        req.into_inner(),
    )
    .await?;
    Success::created(key)
}

#[post("/revoke")]
pub async fn post_revoke(
    pool: web::Data<Arc<PgPool>>,
    req: web::Json<RevokeKeyRequest>,
) -> Res<impl Responder> {
    let key_id = req.key_id;
    let key = service::key::update_key_status(&pool, key_id, "revoked").await?;
    Success::ok(key)
}
