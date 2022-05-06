use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{any, str::FromStr};

#[derive(Debug, Deserialize, Serialize)]
pub enum Frequency {
    Freq2_4GHz,
    Freq5Ghz,
}

#[cfg(target_os = "linux")]
impl FromStr for Frequency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>()? {
            2 => Ok(Self::Freq2_4GHz),
            5 => Ok(Self::Freq5Ghz),
            n => Err(anyhow::anyhow!("Unsupported frequency. {}", n)),
        }
    }
}

#[cfg(target_os = "windows")]
impl FromStr for Frequency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("ac") {
            Ok(Self::Freq2_4GHz)
        } else if s.ends_with('n') {
            Ok(Self::Freq5Ghz)
        } else {
            Err(anyhow::anyhow!("Unsupported frequency."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Wifi {
    // SSID of the network
    pub ssid: String,
    // MAC address of the network
    pub mac: String,
    // channel this network is on
    pub channel: u8,
    // quality of this wifi
    pub quality: f32,
    // frequency
    pub freq: Frequency,
}

#[cfg(target_os = "linux")]
impl FromStr for Wifi {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let mac = lines
            .next()
            .and_then(|line| line.split(": ").last())
            .map(str::to_string)
            .unwrap_or_default();

        let channel = lines
            .next()
            .and_then(|line| line.split(':').last())
            .and_then(|channel_str| channel_str.parse::<u8>().ok())
            .with_context(|| anyhow::anyhow!("No channel found."))?;

        let frequency: Frequency = lines
            .next()
            .and_then(|line| line.split(':').last())
            .and_then(|tok| tok.split('.').next())
            .map(str::parse)
            .with_context(|| anyhow::anyhow!("Couldn't find frequency."))??;

        let quality = lines
            .next()
            .and_then(|line| line.split("Quality=").last())
            .and_then(|tok| tok.split("  ").next())
            .map(|quality| quality.split('/'))
            .map(|mut quality| match (quality.next(), quality.next()) {
                (Some(q1), Some(q2)) => Ok(q1.parse::<f32>()? / q2.parse::<f32>()?),
                _ => Err(anyhow::anyhow!("No quality found.")),
            })
            .with_context(|| anyhow::anyhow!("No quality found."))??;

        // encryption
        lines.next();
        // ESSID
        let ssid = lines
            .next()
            .and_then(|token| token.split("ESSID").last())
            .unwrap_or("")
            .replace('\"', "");

        Ok(Self {
            ssid,
            mac,
            channel,
            quality,
            freq: frequency,
        })
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn into_wifi_vec(_x: &str) -> Result<Vec<Wifi>, anyhow::Error> {
    use itertools::Itertools;

    let mut lines = _x.lines();
    // the first line has the SSID that all APs will have.
    let ssid = lines
        .next()
        .and_then(|tok| tok.split(": ").last())
        .map(str::to_string)
        .unwrap_or_default();

    // Type of network
    lines.next();

    // Authentication
    lines.next();

    // Cryptography
    lines.next();

    let wifis = lines
        .into_iter()
        .chunks(6)
        .into_iter()
        .flat_map(|mut chunk| {
            let mac = chunk
                .next()
                .and_then(|tok| tok.split(": ").last())
                .map(str::to_string)
                .with_context(|| anyhow::anyhow!("Couldn't access quality."))?;

            let quality = chunk
                .next()
                .and_then(|tok| tok.split(": ").last())
                .and_then(|tok| tok.split('%').next())
                .map(f32::from_str)
                .with_context(|| anyhow::anyhow!("Couldn't find quality."))??;

            let freq = chunk
                .next()
                .and_then(|tok| tok.split(": ").last())
                .map(str::parse)
                .with_context(|| anyhow::anyhow!("Couldn't find frequency."))??;

            let channel = chunk
                .next()
                .and_then(|tok| tok.split(": ").last())
                .map(str::parse)
                .with_context(|| anyhow::anyhow!("Couldn't find a channel."))??;

            Ok(Wifi {
                ssid: ssid.clone(),
                mac,
                channel,
                quality: quality / 100f32,
                freq,
            })
        })
        .collect_vec();

    Ok(wifis)
}

#[cfg(target_os = "macos")]
impl TryFrom<&str> for Wifi {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        compile_error!("MacOS support not yet implemented. Give the dev a coffee.")
    }
}
