use std::sync::Arc;

use actix_web::{
    get, web::{self}, Responder
};
use common::{error::Res, http::Success};
use sqlx::PgPool;

use crate::{dtos::usage::KeyUsageRequest, service};

#[get("/usage")]
pub async fn get_usage(
    pool: web::Data<Arc<PgPool>>,
    req: web::Query<KeyUsageRequest>,
) -> Res<impl Responder> {
    let usage_log = service::usage::get_usage_logs(&pool, req.into_inner()).await?;
    Success::ok(usage_log)
}
