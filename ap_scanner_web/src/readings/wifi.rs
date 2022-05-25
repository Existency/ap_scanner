use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Frequency {
    Freq2400MHz(u16),
    Freq5000MHz(u16),
}

impl FromStr for Frequency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s.parse::<u16>()?;

        if val < 2500 {
            Ok(Self::Freq2400MHz(val))
        } else {
            // if the frequency is higher than channel 13
            Ok(Self::Freq5000MHz(val))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Width {
    MHz20,
    MHz40,
    MHz80,
    MHz160,
}

impl FromStr for Width {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split(' ').next().and_then(|val| val.parse::<u8>().ok()) {
            Some(20) => Ok(Self::MHz20),
            Some(40) => Ok(Self::MHz40),
            Some(80) => Ok(Self::MHz80),
            Some(160) => Ok(Self::MHz160),
            _ => Err(anyhow!("Couldn't parse channel width.")),
        }
    }
}

// Alternatively iwlist iface scanning could yield quality levels

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Wifi {
    pub ssid: String,
    pub mac: String,
    pub channel: u8,
    pub signal: f32,
    pub frequency: Frequency,
    pub width: Width,
}
