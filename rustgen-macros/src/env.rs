use crate::object::{from_str, search, Result, Value};
use crate::rustgen_ir::ExprField;

use std::env;
use std::fs::File;
use std::io::Read;

const RUSTGEN_CONFIG_PATH: &str = "RUSTGEN_CONFIG_PATH";

#[derive(Debug)]
pub struct Env(pub Value);

impl Env {
    pub fn new() -> Self {
        let config = env::var(RUSTGEN_CONFIG_PATH).unwrap();
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let config = format!("{}/{}", crate_dir, config);
        let mut file = File::open(config).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let object = from_str(&content).unwrap();
        Env(object)
    }

    pub fn search(&self, field: &ExprField) -> Result<&Value> {
        let value = &self.0;
        search(value, field)
    }
}
