use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Wifi {
    pub ssid: String,
    pub mac: String,
    pub channel: u8,
    pub signal: f32,
    pub frequency: Frequency,
    pub width: Width,
}

impl Wifi {
    pub fn is_2_4g(&self) -> bool {
        matches!(self.frequency, Frequency::Freq2400MHz(_))
    }

    #[allow(dead_code)]
    pub fn is_5g(&self) -> bool {
        matches!(self.frequency, Frequency::Freq5000MHz(_))
    }
}

impl FromStr for Wifi {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let mac = lines
            .next()
            .and_then(|line| line.split('(').next())
            .map(str::to_string)
            .with_context(|| anyhow!("Error parsing BSS."))?;

        // TSF
        lines.next();

        let frequency = lines
            .next()
            .and_then(|line| line.split(' ').last())
            .map(str::parse)
            .with_context(|| anyhow!("Error parsing frequency."))??;

        // beacon interval
        lines.next();

        // capability
        lines.next();

        let signal = lines
            .next()
            .and_then(|line| line.split(' ').nth(1))
            .and_then(|sig| sig.parse().ok())
            .with_context(|| anyhow!("Error parsing signal."))?;

        // last seen
        lines.next();

        // Information elements
        lines.next();

        let ssid = lines
            .next()
            .and_then(|line| line.split(' ').last())
            .map(str::to_string)
            .unwrap_or_default();

        // skip until HT Capabilities
        #[allow(clippy::while_let_on_iterator)]
        while let Some(line) = lines.next() {
            if line.starts_with("\tHT operation") {
                break;
            }
        }

        let channel: u8 = lines
            .next()
            .and_then(|line| line.split(": ").last())
            .map(str::parse)
            .with_context(|| anyhow!("Couldn't parse channel."))??;

        // secondary channel
        lines.next();

        let width: Width = lines
            .next()
            .and_then(|line| line.split(": ").last())
            .map(str::parse)
            .with_context(|| anyhow!("Couldn't parse channel width."))??;

        Ok(Self {
            ssid,
            mac,
            channel,
            signal,
            frequency,
            width,
        })
    }
}
