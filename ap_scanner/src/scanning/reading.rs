use super::{scanner::Scanner, wifi::Wifi};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Each local has the following:
// -
#[derive(Debug, Deserialize, Serialize)]
pub struct Reading {
    // time since epoch in milliseconds
    pub timestamp: u128,
    // identifier of the local this measure was taken
    pub local: String,
    // count of the number of wifi networks found
    pub wifi_count: usize,
    // list of wifi networks found
    pub wifi_list: Vec<Wifi>,
}

impl Reading {
    pub fn new(local: String) -> anyhow::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .with_context(|| anyhow::anyhow!("Failed to get timestamp."))?
            .as_millis();

        let wifi_list = Scanner::scan()?;

        let wifi_count = wifi_list.len();

        Ok(Self {
            timestamp,
            local,
            wifi_count,
            wifi_list,
        })
    }

    pub fn analyze(&self) -> anyhow::Result<()> {
        // analyzes the local networks and outputs to a log file
        // this log file's name is a concatenation of the place and the timestamp
        // such as place_timestamp.txt

        // current suggestions are limited to switching channels

        // given a list of wifi networks distribute them to the appropriate channels
        // and output to a log file

        self.wifi_list.iter().for_each(|_wifi| {
            todo!("analyze wifi networks")
            // non overlapping 2.4 GHz channels:
            // 1 6 11

            // non overlapping 5 GHz channels:
            // 36 40 44 48 52 56 60 64
            // 100 104 108 112 116 120 124 128 136 140 144
            // 149 153 157 161 165
        });

        Ok(())
    }

    pub fn deserialize(path: String) -> anyhow::Result<Self> {
        let file = std::fs::File::open(&path)?;

        serde_json::from_reader(file)
            .with_context(|| format!("Failed to load measure from {:?}", &path))
    }

    pub fn serialize(&self, path: String) -> anyhow::Result<()> {
        let file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to open file {:?}", &path))?;

        serde_json::to_writer_pretty(file, self)
            .with_context(|| format!("Failed to save measure to {:?}", &path))
    }
}