/// data transfer objects

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOut {
    pub name: String,
    pub message: String,
    pub stack: String,
}
