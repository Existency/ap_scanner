use super::{scanner::Scanner, wifi::Wifi};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Each local has the following:
// -
#[derive(Debug, Deserialize, Serialize)]
pub struct Measure {
    // time since epoch in milliseconds
    pub timestamp: u128,
    // identifier of the local this measure was taken
    pub local: String,
    // count of the number of wifi networks found
    pub wifi_count: u32,
    // list of wifi networks found
    pub wifi_list: Vec<Wifi>,
}

impl Measure {
    pub fn new<T: Into<String>>(local: T) -> anyhow::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .with_context(|| anyhow::anyhow!("Failed to get timestamp."))?
            .as_millis();

        Ok(Self {
            timestamp,
            local: local.into(),
            wifi_count: 0,
            wifi_list: Scanner::scan()?,
        })
    }

    // Loads a measure from a json file
    pub fn from_json(path: String) -> anyhow::Result<Self> {
        let file = std::fs::File::open(&path)?;

        serde_json::from_reader(file)
            .with_context(|| format!("Failed to load measure from {:?}", &path))
    }

    // Saves a measure to a json file
    pub fn to_json(&self, path: String) -> anyhow::Result<()> {
        // create the file
        let file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to open file {:?}", &path))?;

        serde_json::to_writer_pretty(file, self)
            .with_context(|| format!("Failed to save measure to {:?}", &path))
    }
}
