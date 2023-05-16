use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Encoding<T> {
    pub encoding: Vec<usize>,
    pub tokens: Vec<T>,
}
