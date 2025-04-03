use common::error::{AppError, Res};
use crate::models::log::Log;
use sqlx::{Executor, Postgres};

pub async fn get_report<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    method: Option<String>,
    code: Option<i32>,
    path: Option<String>,
) -> Res<Vec<Log>> {
    let mut query_conditions = Vec::new();
    let mut params = Vec::new();
    let mut param_count = 1;

    let mut query_base = "SELECT * FROM logs".to_string();

    if let Some(method) = method {
        query_conditions.push(format!("method = ${}", param_count));
        params.push(method.clone());
        param_count += 1;
    }
    if let Some(status_code) = code {
        query_conditions.push(format!("status_code = ${}::INTEGER", param_count));
        params.push(status_code.to_string());
        param_count += 1;
    }
    if let Some(path) = path {
        query_conditions.push(format!("path = ${}", param_count));
        params.push(path.clone());
    }

    if !query_conditions.is_empty() {
        query_base.push_str(" WHERE ");
        query_base.push_str(&query_conditions.join(" AND "));
    }

    let mut query = sqlx::query_as::<_, Log>(&query_base);

    for param in params {
        query = query.bind(param);
    }

    query.fetch_all(executor).await.map_err(AppError::from)
}

pub async fn insert_log<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    log: Log,
) -> Res<()> {
    sqlx::query(
        "INSERT INTO logs (timestamp, method, path, status_code, user_id, params, request_body, response_body, ip_address, user_agent) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(log.timestamp)
    .bind(&log.method)
    .bind(&log.path)
    .bind(log.status_code)
    .bind(log.user_id)
    .bind(log.params)
    .bind(log.request_body)
    .bind(log.response_body)
    .bind(log.ip_address)
    .bind(log.user_agent)
    .execute(executor)
    .await
    .map_err(AppError::from)?;

    Ok(())
}
