use wasm_bindgen::prelude::*;

use crate::{
    collapse_encoding, create_encoding_graph, encode_with_run_compression, null_encoding, Encoding,
};

#[wasm_bindgen]
pub fn create_encoding_graph_wasm(encoding_str: &str) -> String {
    let encoding: Encoding<String> = serde_json::from_str(&encoding_str).unwrap();
    format!("{}", create_encoding_graph(&encoding).unwrap())
}

#[wasm_bindgen]
pub fn null_encoding_wasm(text: &str) -> String {
    serde_json::to_string(&null_encoding(text)).unwrap()
}

#[wasm_bindgen]
pub fn encode_with_run_compression_wasm(encoding_str: &str) -> Option<String> {
    let mut encoding = serde_json::from_str(&encoding_str).unwrap();
    let res = encode_with_run_compression(&mut encoding);
    if res {
        Some(serde_json::to_string(&encoding).unwrap())
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn collapse_encoding_edge_wasm(encoding_str: &str, n1: usize, n2: usize) -> String {
    let mut encoding: Encoding<String> = serde_json::from_str(&encoding_str).unwrap();
    collapse_encoding(&mut encoding, &[n1, n2]);
    serde_json::to_string(&encoding).unwrap()
}
