use actix_web::body::{self, BoxBody, MessageBody};
use actix_web::dev::Payload;
use actix_web::web::{self, Bytes};
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use actix_web::{HttpMessage, HttpResponse, ResponseError};
use chrono::Utc;
use colored::Colorize;
use common::env_config::Config;
use common::jwt::get_jwt_claims_or_error;
use common::key::get_key_claims_or_error;
use db::models::log::Log;
use futures::StreamExt;
use futures::future::{LocalBoxFuture, Ready, ready};
use log::{debug, info};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::types::ipnetwork::IpNetwork;
use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub struct LoggerMiddleware {}

impl LoggerMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = LoggerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddlewareService {
            service: Arc::new(service),
        }))
    }
}

pub struct LoggerMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Common request info
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query_string = req.query_string().to_string();

        // IP
        let ip_str = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let ip_address = IpNetwork::from_str(&ip_str)
            .unwrap_or_else(|_| IpNetwork::from_str("0.0.0.0").unwrap());

        // Agent
        let user_agent = req
            .headers()
            .get("User-Agent")
            .map(|ua| ua.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        let config = &***req.app_data::<web::Data<Arc<Config>>>().unwrap().clone();
        let console_logging_enabled = config.console_logging_enabled;
        let srv = Arc::clone(&self.service);

        Box::pin(async move {
            // Jwt claims
            let jwt_claims = get_jwt_claims_or_error(&req).ok();
            let mut user_id = jwt_claims.as_ref().map(|c| c.user_id);

            // Key claims
            let key_claims = get_key_claims_or_error(&req).ok();
            let key_id = key_claims.as_ref().map(|c| c.key_id);
            if user_id.is_none() {
                user_id = key_claims.as_ref().map(|c| c.user_id);
            }

            // Get postgres pool
            let pool = &***req.app_data::<web::Data<Arc<PgPool>>>().unwrap().clone();

            // Copy request body from payload and reconstruct it
            let mut payload = req.take_payload();
            let body_bytes = extract_body(&mut payload).await?;
            let request_body = if !body_bytes.is_empty() {
                serde_json::from_slice::<Value>(&body_bytes).unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            let new_stream: Pin<
                Box<dyn futures::Stream<Item = Result<Bytes, actix_web::error::PayloadError>>>,
            > = futures::stream::once(async move {
                Ok::<Bytes, actix_web::error::PayloadError>(body_bytes)
            })
            .boxed();
            req.set_payload(Payload::from(new_stream));

            // Call next services
            let res = srv.call(req).await?;

            // Get response status
            let status = res.status().clone();
            let status_code = res.status().as_u16() as i32;
            let timestamp = Utc::now();

            // Get response params
            let params_json = if !query_string.is_empty() {
                let mut params_map = HashMap::new();
                for pair in query_string.split('&') {
                    if let Some(pos) = pair.find('=') {
                        let key = &pair[0..pos];
                        let value = &pair[pos + 1..];
                        params_map.insert(key.to_string(), json!(value));
                    } else {
                        params_map.insert(pair.to_string(), json!(true));
                    }
                }
                json!(params_map)
            } else {
                json!({})
            };

            // Copy response body and reconstruct response
            let (req, res) = res.into_parts();
            let headers = res.headers().clone();
            let res_body = res.into_body();
            let response_body_bytes = body::to_bytes(res_body).await?;
            let response_body =
                serde_json::from_slice::<Value>(&response_body_bytes).unwrap_or(Value::Null);
            let mut new_res = HttpResponse::build(status);
            for (key, value) in headers.iter() {
                new_res.insert_header((key.clone(), value.clone()));
            }
            let new_res = new_res.body(response_body_bytes);
            let res = ServiceResponse::new(req, new_res);

            // Log to console
            if console_logging_enabled {
                let colored_status = match status_code {
                    200..=299 => status_code.to_string().green(),
                    300..=399 => status_code.to_string().yellow(),
                    400..=499 => status_code.to_string().bright_red(),
                    _ => status_code.to_string().red(),
                };

                let colored_method = match method.as_str() {
                    "GET" => method.blue(),
                    "POST" => method.yellow(),
                    "PUT" => method.purple(),
                    "DELETE" => method.red(),
                    _ => method.normal(),
                };

                info!(
                    "[{}] {} {} {} user_id={} params={}",
                    colored_status,
                    colored_method,
                    path.bright_white(),
                    format!("({:?}ms)", 0).bright_black(),
                    user_id
                        .map_or("None".to_string(), |id| id.to_string())
                        .bright_blue(),
                    params_json.to_string().bright_cyan(),
                );

                if let Some(ref body) = request_body.as_object() {
                    if !body.is_empty() {
                        debug!(
                            "  Request: {}",
                            serde_json::to_string(&request_body)
                                .unwrap_or_default()
                                .bright_green()
                        );
                    }
                }

                if status_code >= 400
                    || (response_body.is_object() && !response_body.as_object().unwrap().is_empty())
                {
                    debug!(
                        "  Response: {}",
                        serde_json::to_string(&response_body)
                            .unwrap_or_default()
                            .bright_yellow()
                    );
                }
            }

            // Insert into database
            db::log::insert_log(
                pool,
                Log {
                    id: Uuid::nil(), // auto-generated
                    timestamp: timestamp.naive_utc(),
                    method,
                    path,
                    status_code,
                    user_id,
                    params: Some(params_json),
                    key_id,
                    request_body: Some(request_body),
                    response_body: Some(response_body),
                    ip_address,
                    user_agent,
                },
            )
            .await?;

            Ok(res)
        })
    }
}

async fn extract_body(payload: &mut Payload) -> Result<Bytes, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }
    Ok(body.freeze())
}
