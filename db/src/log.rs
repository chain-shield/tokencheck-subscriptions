use crate::{dtos::log::ReportFilter, models::log::Log};
use common::error::{AppError, Res};
use sqlx::{Executor, Postgres, QueryBuilder};

pub async fn get_report<'e, E>(executor: E, filter: ReportFilter) -> Res<Vec<Log>>
where
    E: Executor<'e, Database = Postgres>,
{
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM logs");
    let mut conditions_added = false;

    // Helper to add WHERE or AND
    let mut add_condition_separator = |qb: &mut QueryBuilder<Postgres>| {
        if !conditions_added {
            qb.push(" WHERE ");
            conditions_added = true;
        } else {
            qb.push(" AND ");
        }
    };

    if let Some(user_id) = filter.user_id {
        add_condition_separator(&mut qb);
        qb.push("user_id = ").push_bind(user_id);
    }

    if let Some(key_id) = filter.key_id {
        add_condition_separator(&mut qb);
        qb.push("key_id = ").push_bind(key_id);
    }

    if let Some(method) = filter.method {
        add_condition_separator(&mut qb);
        qb.push("method = ").push_bind(method);
    }

    if let Some(status_code) = filter.code {
        add_condition_separator(&mut qb);
        qb.push("status_code = ").push_bind(status_code);
    }

    if let Some(path) = filter.path {
        add_condition_separator(&mut qb);
        qb.push("path LIKE ").push_bind(format!("%{}%", path));
    }

    if let Some(ending_before) = filter.ending_before {
        add_condition_separator(&mut qb);
        qb.push("timestamp < ").push_bind(ending_before);
    }

    if let Some(starting_after) = filter.starting_after {
        add_condition_separator(&mut qb);
        qb.push("timestamp > ").push_bind(starting_after);
    }

    if let Some(limit) = filter.limit {
        qb.push(" LIMIT ").push_bind(limit);
    }

    let query = qb.build_query_as::<Log>(); // Build the final query

    query.fetch_all(executor).await.map_err(AppError::from) // Execute
}

pub async fn insert_log<'e, E: Executor<'e, Database = Postgres>>(
    executor: E,
    log: Log,
) -> Res<()> {
    sqlx::query(
        "INSERT INTO logs (timestamp, method, path, status_code, user_id, params, key_id, request_body, response_body, ip_address, user_agent) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(log.timestamp)
    .bind(&log.method)
    .bind(&log.path)
    .bind(log.status_code)
    .bind(log.user_id)
    .bind(log.params)
    .bind(log.key_id)
    .bind(log.request_body)
    .bind(log.response_body)
    .bind(log.ip_address)
    .bind(log.user_agent)
    .execute(executor)
    .await
    .map_err(AppError::from)?;

    Ok(())
}
