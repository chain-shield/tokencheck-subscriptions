use uuid::Uuid;

pub struct ReportFilter {
    pub user_id: Option<Uuid>,
    pub key_id: Option<Uuid>,
    pub method: Option<String>,
    pub code: Option<i32>,
    pub path: Option<String>,
    pub limit: Option<i32>,
    pub ending_before: Option<String>,
    pub starting_after: Option<String>,
}