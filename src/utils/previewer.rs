use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::Path;

pub struct JSONPreviewer {
    data: String,
}

impl JSONPreviewer {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn to_hashmap(&self) -> HashMap<String, serde_json::Value> {
        let data: HashMap<String, serde_json::Value> = serde_json::from_str(&self.data).unwrap();
        data
    }

    pub fn to_json(&self) -> String {
        let data: HashMap<String, serde_json::Value> = self.to_hashmap();
        serde_json::to_string(&data).unwrap()
    }
}

pub fn read_json_file(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    Ok(data)
}
