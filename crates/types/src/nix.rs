use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Nix {
    pub id: String,
}
