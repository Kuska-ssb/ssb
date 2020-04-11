#[derive(Debug, Serialize, Deserialize)]
pub struct LatestOut {
    pub id: String,
    pub sequence: u64,
    pub ts: f64,
}
