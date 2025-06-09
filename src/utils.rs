use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct Record {
    pub id: Uuid,
    pub region: String,
    pub municipality: String,
    pub company: String,
    pub phone: String,
    pub contact: String,
    pub total_order: u32,
    pub recent_order: u32,
}
