use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
}
