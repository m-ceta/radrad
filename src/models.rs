use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: Option<String>,
    pub af: bool,
    pub tf: bool,
}
