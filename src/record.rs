use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct Account {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    ty: String,
    client: u16,
    tx: u32,
    amount: f64,
}

