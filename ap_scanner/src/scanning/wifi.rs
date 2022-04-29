use serde::{Deserialize, Serialize};
use std::str::FromStr;
// - Frequência, Canais, Potência

#[derive(Debug, Deserialize, Serialize)]
pub struct Wifi {
    // SSID of the network
    pub ssid: String,
    // MAC address of the network
    pub mac: String,
    // frequency of the network
    pub freq: f32,
    // channel this network is on
    pub channel: u8,
    // quality of this wifi
    pub quality: f32,
    // signal level in dBm
    pub signal: i32,
}

#[cfg(target_os = "linux")]
impl TryFrom<&str> for Wifi {
    type Error = anyhow::Error;
    // 0xE00101B0

    fn try_from(value: &str) -> anyhow::Result<Self> {
        let mut lines = value.lines();

        // need to check whether we have a valid input

        let mac = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("No mac found."))?
            .split("Address: ")
            .last()
            .ok_or_else(|| anyhow::anyhow!("No mac found."))?;

        let channel = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("No channel found."))?
            .split("Channel:")
            .last()
            .ok_or_else(|| anyhow::anyhow!("No channel found."))?
            .parse::<u8>()?;

        let frequency = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("No frequency found."))?
            .split("Frequency:")
            .last()
            .ok_or_else(|| anyhow::anyhow!("No frequency found."))?
            .split(" GHz")
            .next()
            .ok_or_else(|| anyhow::anyhow!("No frequency found."))?;

        let freq = f32::from_str(frequency)?;

        let mut tmp = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("No quality found."))?
            .split("Quality=")
            .last()
            .ok_or_else(|| anyhow::anyhow!("No quality found."))?
            .split("  ");

        let mut quality = tmp
            .next()
            .ok_or_else(|| anyhow::anyhow!("No quality found."))?
            .split('/');

        let q1 = quality
            .next()
            .ok_or_else(|| anyhow::anyhow!("No quality found."))?
            .parse::<f32>()?;

        let q2 = quality
            .next()
            .ok_or_else(|| anyhow::anyhow!("No quality found."))?
            .parse::<f32>()?;

        let tmp_sig = tmp.collect::<String>();
        let signal = tmp_sig
            .split("level=")
            .last()
            .ok_or_else(|| anyhow::anyhow!("No signal found."))?
            .split(" dBm")
            .next()
            .ok_or_else(|| anyhow::anyhow!("No signal found."))?
            .parse::<i32>()?;

        // encryption
        lines.next();
        // ESSID
        let ssid = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("No ssid found."))?
            .split("ESSID:")
            .last()
            .unwrap_or("")
            .replace('\"', "");

        Ok(Self {
            ssid,
            mac: mac.to_string(),
            freq,
            channel,
            quality: (q1 / q2) as f32,
            signal,
        })
    }
}

#[cfg(target_os = "windows")]
impl TryFrom<&str> for Wifi {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        anyhow::anyhow!("Windows support not yet implemented. Give the dev a coffee.")
    }
}

#[cfg(target_os = "macos")]
impl TryFrom<&str> for Wifi {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        anyhow::anyhow!("MacOS support not yet implemented. Give the dev a coffee.")
    }
}
