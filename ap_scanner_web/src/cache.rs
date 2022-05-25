use serde::{Deserialize, Serialize};

use crate::readings::{Reading, Suggestion};
use std::{collections::HashMap, io::Write, sync::Arc, sync::Mutex};

#[derive(Serialize, Deserialize)]
struct _InnerValues {
    pub ssid: String,
    pub file: String,
    pub suggestion: Suggestion,
}

#[derive(Serialize, Deserialize)]
struct _DiskCache(HashMap<String, _InnerValues>);

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<_DiskCache>> = Arc::new(Mutex::new(_DiskCache(HashMap::new())));
}

pub struct Cache;

impl Cache {
    pub fn create_dir() -> std::io::Result<()> {
        // create the upload directory
        std::fs::create_dir_all("upload")?;
        // create the cache file
        std::fs::File::create("upload/cache")?;

        Ok(())
    }

    /// Receives a Vector of tuples of SSID, MAC and Suggestions. Also receives the filename.
    /// The filename is where the scan was saved.
    /// The SSID and MAC are the keys of the cache.
    /// The suggestions are the values of the cache.
    pub fn insert_into_cache(reading: Reading, file: &str) {
        // insert into cache
        let data = reading
            .wifi_2_4_ghz
            .iter()
            .chain(reading.wifi_5_ghz.iter())
            .map(|(_, pair)| {
                pair.iter()
                    .map(|(w, s)| (w.ssid.clone(), w.mac.clone(), s.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut cache = CACHE.lock().unwrap();

        for (ssid, mac, suggestion) in data {
            cache.0.insert(
                mac,
                _InnerValues {
                    ssid,
                    file: file.to_string(),
                    suggestion: suggestion.clone(),
                },
            );
        }
    }

    /// Given a SSID and MAC, returns the filename and the suggestion.
    pub fn get_suggestion(ssid: &str, mac: &str) -> Option<Suggestion> {
        let cache = CACHE.lock().unwrap();

        cache
            .0
            .get(mac)
            .map(|val| {
                if val.ssid.eq(ssid) {
                    Some(val.suggestion.clone())
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn from_file(path: &str) {
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(disk_cache) = serde_json::from_reader(file) {
                let mut cache = CACHE.lock().unwrap();
                cache.0 = disk_cache;
            }
        }
    }

    pub fn to_file(path: &str) -> std::io::Result<()> {
        let cache = &CACHE.lock().unwrap().0;
        let val = serde_json::to_string(cache).unwrap();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&val.as_bytes())?;
        Ok(())
    }
}
