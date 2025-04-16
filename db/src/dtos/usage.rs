use uuid::Uuid;

pub struct KeyUsageRequest {
    pub key_id: Uuid,
    pub limit: u32
}