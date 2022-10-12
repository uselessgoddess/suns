use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Info {
    link: String,
    row: usize,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Spec {
    ISiT,
}

impl Spec {
    fn info(&self) -> Info {}
}
