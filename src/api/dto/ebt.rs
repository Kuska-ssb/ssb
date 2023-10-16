#[derive(Debug, Serialize, Deserialize)]
pub struct EbtReplicate {
    pub version: u16,
    pub format: String,
}

impl Default for EbtReplicate {
    fn default() -> Self {
        Self {
            version: 3,
            format: "classic".to_string(),
        }
    }
}
