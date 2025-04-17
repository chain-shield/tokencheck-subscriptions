use actix_web::{HttpResponse, Responder};
use serde::Serialize;

use super::error::Res;

pub struct Success;
impl Success {
    pub fn created<T: Serialize>(body: T) -> Res<impl Responder> {
        Result::Ok(HttpResponse::Created().json(body))
    }
    pub fn ok<T: Serialize>(body: T) -> Res<impl Responder> {
        Result::Ok(HttpResponse::Ok().json(body))
    }
}
