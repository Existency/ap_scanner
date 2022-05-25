use super::wifi::Wifi;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Channel type alias
type Channel = u8;

#[derive(Debug, Deserialize, Serialize)]
pub struct Reading {
    // time since epoch in milliseconds
    pub timestamp: u128,
    // identifier of the local this measure was taken
    pub local: String,
    // Hashmap representing the distribution of 2.4 GHz AP
    pub wifi_2_4_ghz: HashMap<Channel, Vec<(Wifi, Suggestion)>>,
    // Hashmap representing the distribution of 5 GHz AP
    pub wifi_5_ghz: HashMap<Channel, Vec<(Wifi, Suggestion)>>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Suggestion {
    Suggestion2g(Channel),
    Suggestion5g(Suggestions5G),
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Suggestions5G {
    pub ndfs_20: Channel,
    pub dfs_20: Channel,
    pub ndfs_40: Channel,
    pub dfs_40: Channel,
    pub ndfs_80: Channel,
    pub dfs_80: Channel,
    pub dfs_160: Channel,
}
