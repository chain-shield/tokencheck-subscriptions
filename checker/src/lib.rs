use std::time::Duration;
use actix_web::{post, web, Responder};
use common::{error::Res, http::Success};
use tokio::time::sleep;

/// Test function that simulates checking tokens
#[post("/check-token")]
async fn check_tokens() -> Res<impl Responder> {
    log::info!("Start token checker");
    sleep(Duration::from_millis(1000)).await;
    log::info!("Stop token checker");
    Success::ok(())
}

pub fn mount_checker() -> actix_web::Scope {
    web::scope("/checker")
        .service(check_tokens)
}