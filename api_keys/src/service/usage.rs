use common::error::{AppError, Res};
use db::dtos::log::ReportFilter;
use sqlx::PgPool;

use crate::dtos::usage::{KeyUsageRequest, UsageResponse};

/// Retrieves usage logs based on the provided request.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `req` - The request containing the filters for retrieving usage logs.
///
/// # Returns
///
/// A `Result` containing a vector of `UsageResponse` objects or an `AppError` if an error occurs.
pub async fn get_usage_logs(pool: &PgPool, req: KeyUsageRequest) -> Res<Vec<UsageResponse>> {
    // Check if user_id or key_id is set
    if req.user_id.is_none() && req.key_id.is_none() {
        return Err(AppError::BadRequest(
            "Both user id and key id were not set. At least one value must be set.".to_string(),
        ));
    }

    // Get logs from database
    let logs = db::log::get_report(
        pool,
        ReportFilter {
            user_id: req.user_id,
            key_id: req.key_id,
            method: None,
            code: None,
            path: Some(format!("/v1")),
            limit: Some(req.limit),
            starting_after: req.starting_after,
            ending_before: req.ending_before,
        },
    )
    .await?;

    // Map logs to UsageResponse objects
    Ok(logs
        .iter()
        .map(|log| UsageResponse {
            name: log.key_id.unwrap_or_default().to_string(),
            date: log.timestamp,
            path: log.path.clone(),
        })
        .collect())
}
