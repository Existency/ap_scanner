use super::scanner::Scanner;
use super::wifi::Wifi;
use anyhow::Context;
use itertools::Itertools;
use rand::{
    distributions::{Slice, WeightedIndex},
    prelude::Distribution,
    Rng,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Reading {
    // time since epoch in milliseconds
    pub timestamp: u128,
    // identifier of the local this measure was taken
    pub local: String,
    // Hashmap representing the distribution of 2.4 GHz AP
    pub wifi_2_4_ghz: HashMap<u8, Vec<(Wifi, Suggestion)>>,
    // Hashmap representing the distribution of 5 GHz AP
    pub wifi_5_ghz: HashMap<u8, Vec<(Wifi, Suggestion)>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Suggestion {
    Suggestion2g(u8),
    Suggestion5g(Suggestions5G),
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Suggestion::Suggestion2g(v) => *v,
            Suggestion::Suggestion5g(v) => v.ndfs_20,
        };

        f.write_str(&format!("{}", out))
    }
}

impl Suggestion {
    pub fn default_5g() -> Self {
        Self::Suggestion5g(Suggestions5G::default())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Suggestions5G {
    pub ndfs_20: u8,
    pub dfs_20: u8,
    pub ndfs_40: u8,
    pub dfs_40: u8,
    pub ndfs_80: u8,
    pub dfs_80: u8,
    pub dfs_160: u8,
}

impl Reading {
    pub fn new(local: String) -> anyhow::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .with_context(|| anyhow::anyhow!("Failed to get timestamp."))?
            .as_millis();

        let wifi_list = Scanner::scan()?;

        let (left, right): (Vec<_>, Vec<_>) = wifi_list.into_iter().partition(Wifi::is_2_4g);

        // 2.4 GHz
        let mut wifi_2_4_ghz = left
            .into_iter()
            .map(|wifi| {
                let channel = wifi.channel;
                (wifi, Suggestion::Suggestion2g(channel))
            })
            .into_group_map_by(|tuple| tuple.0.channel);

        let count_2g = wifi_2_4_ghz.values().map(|v| v.len()).sum::<usize>();

        let one = count_2g - wifi_2_4_ghz.get(&1u8).map(|vc| vc.len()).unwrap_or(0);
        let six = count_2g - wifi_2_4_ghz.get(&6u8).map(|vc| vc.len()).unwrap_or(0);
        let eleven = count_2g - wifi_2_4_ghz.get(&11u8).map(|vc| vc.len()).unwrap_or(0);

        let weights = [(1u8, one), (6, six), (11, eleven)];
        let distr_24 = WeightedIndex::new(weights.iter().map(|i| i.1))?;

        wifi_2_4_ghz.iter_mut().for_each(|x| {
            if ![1, 6, 11].contains(x.0) {
                x.1.iter_mut().for_each(|x| {
                    x.1 = Suggestion::Suggestion2g(
                        weights[distr_24.sample(&mut rand::thread_rng())].0,
                    );
                });
            }
        });

        // Fair warning to all those who gaze upon this wretched code.
        //
        // The code that lies below is untested, poorly written and should never see the light of day.
        //
        // Take everything you see with a truckload of salt and some off brand tequila. You've been warned.
        //
        // https://www.smallnetbuilder.com/wireless/wireless-features/33210-160-mhz-wi-fi-channels-friend-or-foe?start=1
        // https://www.tech21century.com/best-wifi-channels-for-your-router/
        // https://www.ekahau.com/blog/channel-planning-best-practices-for-better-wi-fi/

        // 5 GHz
        let mut wifi_5_ghz = right
            .into_iter()
            .map(|wifi| (wifi, Suggestion::default_5g()))
            .into_group_map_by(|x| x.0.channel);

        let count_5g = wifi_5_ghz.values().map(|v| v.len()).sum();

        let distr_ndfs20 = &[36u8, 40, 44, 48, 149, 153, 157, 161, 165];
        let mut ndfs20 = get_distr_vec(distr_ndfs20, count_5g)?.into_iter();

        let distr_dfs20 = &[
            36u8, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136,
            140, 144, 149, 153, 157, 161, 165,
        ];
        let mut dfs20 = get_distr_vec(distr_dfs20, count_5g)?.into_iter();

        let non_dfs_distr_40mhz = &[38u8, 46, 151, 159];
        let mut ndfs40 = get_distr_vec(non_dfs_distr_40mhz, count_5g)?.into_iter();

        let dfs_distr_40mhz = &[38u8, 46, 54, 62, 102, 110, 118, 126, 134, 142, 151, 159];
        let mut dfs40 = get_distr_vec(dfs_distr_40mhz, count_5g)?.into_iter();

        let non_dfs_distr_80mhz = &[42u8, 155];
        let mut ndfs80 = get_distr_vec(non_dfs_distr_80mhz, count_5g)?.into_iter();

        let dfs_distr_80mhz = &[42u8, 58, 106, 122, 138, 155];
        let mut dfs80 = get_distr_vec(dfs_distr_80mhz, count_5g)?.into_iter();

        // 160mhz is automatically DFS
        let distr_160mhz = &[50u8, 114];
        let mut dfs160 = get_distr_vec(distr_160mhz, count_5g)?.into_iter();

        wifi_5_ghz.iter_mut().for_each(|x| {
            x.1.iter_mut().for_each(|x| {
                x.1 = Suggestion::Suggestion5g(Suggestions5G {
                    ndfs_20: ndfs20.next().expect("ndfs20"),
                    dfs_20: dfs20.next().expect("dfs20"),
                    ndfs_40: ndfs40.next().expect("ndfs40"),
                    dfs_40: dfs40.next().expect("dfs40"),
                    ndfs_80: ndfs80.next().expect("ndfs80"),
                    dfs_80: dfs80.next().expect("dfs80"),
                    dfs_160: dfs160.next().expect("dfs160"),
                })
            })
        });

        Ok(Self {
            timestamp,
            local,
            wifi_2_4_ghz,
            wifi_5_ghz,
        })
    }

    /// Outputs a human friendly result to screen.
    pub fn output_analysis(&self) -> anyhow::Result<()> {
        let count_2g = self.wifi_2_4_ghz.values().map(|v| v.len()).sum::<usize>();
        let count_5g = self.wifi_5_ghz.values().map(|v| v.len()).sum::<usize>();

        println!(
            "AP Scanner 2022\nNumber of 2.4GHz networks: {}\nNumber of 5GHz networks: {}",
            count_2g, count_5g
        );

        println!("Suggestions for Wifi 2.4GHz networks.");
        // print the suggestions only
        self.wifi_2_4_ghz.iter().for_each(|x| {
            x.1.iter().for_each(|pair| {
                if pair.0.channel != *x.0 {
                    println!(
                        "Wifi network with SSID and MAC: {}, {}.\n\tCurrent channel: {}.\n\tSuggested change:{:?}",
                        &pair.0.ssid, &pair.0.mac, &pair.0.channel, &pair.1
                    )
                }
            })
        });

        println!("The suggested distributions for Wifi 5GHz networks.");
        // HashMap<u8, (Wifi, Suggestion)> -> 6 x (Wifi, u8)

        self.wifi_5_ghz.iter().for_each(|x| {
            x.1.iter().for_each(|pair| {
                println!("Wifi network with SSID and MAC: {}, {}.\n\tSuggested Channels per channel width:", pair.0.ssid, pair.0.mac);
                if let Suggestion::Suggestion5g(sug) = &pair.1 {
                    println!("\t\t20MHz: {}", sug.ndfs_20);
                    println!("\t\tDFS 20MHz: {}", sug.dfs_20);
                    
                    println!("\t\t40MHz: {}", sug.ndfs_40);
                    println!("\t\tDFS 40MHz: {}", sug.dfs_40);
                    
                    println!("\t\t80MHz: {}", sug.ndfs_80);
                    println!("\t\tDFS 80MHz: {}", sug.dfs_80);
                    
                    println!("\t\t160MHz: {}", sug.ndfs_20);
                }
            })
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

fn get_distr_vec(values: &[u8], count: usize) -> anyhow::Result<Vec<u8>> {
    let distribution = Slice::new(values)?;
    let rng = rand::thread_rng();

    Ok(rng
        .sample_iter(&distribution)
        .take(count)
        .cloned()
        .collect_vec())
}
