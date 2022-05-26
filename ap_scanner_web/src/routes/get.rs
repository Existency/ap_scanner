use std::borrow::Cow;

use crate::{readings::Reading, Cache, HOST, SCAN_PATH};
use rocket::tokio::fs::File;
use serde::Serialize;
use walkdir::WalkDir;

use crate::readings::ReadingID;

/// Fetches a list of JSON files inside the scans folder.
fn get_scans() -> Vec<String> {
    WalkDir::new(SCAN_PATH.as_str())
        .into_iter()
        .filter_map(|maybe_entry| {
            maybe_entry.ok().and_then(|dir_entry| {
                dir_entry.file_type().is_file().then(|| {
                    let file_name = dir_entry.file_name().to_string_lossy();
                    file_name
                        .ends_with(".json")
                        .then(|| file_name.trim_end_matches(".json").to_owned())
                })
            })
        })
        .flatten()
        .collect()
}

/// Retrieves a list of JSON files within the scans folder and sends it to the user.
#[get("/index")]
pub async fn index() -> String {
    let scans = get_scans();
    let count = scans.len();

    let mut output = String::new();

    output.push_str("AP Scanner Index\n");

    let sclen = scans.len();

    if sclen > 0 {
        let scan = if sclen == 1 { "scan" } else { "scans" };

        output.push_str(
            format!(
                "There's currently {count} {scan} available.\n\nScans\n",
                count = count,
                scan = scan
            )
            .as_str(),
        );
        for scan in scans {
            output.push_str(&format!("{host}/{id}\n", host = HOST.as_str(), id = scan))
        }
    }

    output
}

/// Generates a String containing the data the user requested.
/// The generated data is human readable.
#[get("/<ssid>/<mac>")]
pub async fn suggestion(ssid: &str, mac: &str) -> Option<String> {
    let mut output = String::new();

    output.push_str("AP Scanner Suggestion\n");
    output.push_str(format!("SSID: {}\n", ssid).as_str());
    output.push_str(format!("MAC: {}\n", mac).as_str());

    if let Some(suggestion) = Cache::get_suggestion(ssid, mac) {
        output.push_str(format!("Suggestion: {:?}\n", suggestion).as_str());
    } else {
        output.push_str("No suggestion available.\n");
    }

    Some(output)
}

#[derive(Serialize)]
struct RawScan<'a> {
    ssid: Cow<'a, str>,
    mac: Cow<'a, str>,
    suggestion: String,
}

/// Generates a JSON with the necessary data and serves it.
/// This differs from `suggestion` in the fact that the data isn't meant to be human readable.
#[get("/<ssid>/<mac>/raw")]
pub async fn suggestion_raw(ssid: &str, mac: &str) -> Option<String> {
    let suggestion = Cache::get_suggestion(ssid, mac).and_then(|cache| {
        let fodasse =
            serde_json::to_string(&cache).unwrap_or("No suggestion available.".to_string());
        Some(fodasse)
    })?;

    let output = RawScan {
        ssid: Cow::Borrowed(ssid),
        mac: Cow::Borrowed(mac),
        suggestion,
    };

    Some(serde_json::to_string(&output).unwrap_or("No suggestion available.".to_string()))
}

/// Retrieves a suggestion for a specific ssid/mac device in a file specified by the client in a json format.
/// If multiple matches for the same SSID/MAC are found, returns the first one.
#[get("/<id>/<ssid>/<mac>")]
pub async fn file_suggestion(id: ReadingID<'_>, ssid: &str, mac: &str) -> Option<String> {
    let file = std::fs::File::open(id.path()).ok()?;
    let reading: Reading = serde_json::from_reader(&file).ok()?;

    let kv_match = reading
        .wifi_2_4_ghz
        .iter()
        .chain(reading.wifi_5_ghz.iter())
        .filter_map(|(_, pair)| {
            pair.iter()
                .filter_map(|(w, s)| {
                    if w.ssid.eq(ssid) && w.mac.eq(mac) {
                        let suggestion = serde_json::to_string(s)
                            .unwrap_or("Couldn't parse the suggestion.".into());
                        Some(RawScan {
                            ssid: Cow::Borrowed(ssid),
                            mac: Cow::Borrowed(mac),
                            suggestion,
                        })
                    } else {
                        None
                    }
                })
                .next()
        })
        .next()?;

    Some(format!(
        "AP Scanner Suggestion\nSSID: {ssid}\nMAC: {mac}\nSuggestion: {suggestion}",
        ssid = kv_match.ssid,
        mac = kv_match.mac,
        suggestion = kv_match.suggestion,
    ))
}

/// Retrieves a suggestion for a specific ssid/mac device in a file specified by the client in a json format.
#[get("/<id>/<ssid>/<mac>/raw")]
pub async fn file_suggestion_raw(id: ReadingID<'_>, ssid: &str, mac: &str) -> Option<String> {
    let file = std::fs::File::open(id.path()).ok()?;
    let reading: Reading = serde_json::from_reader(&file).ok()?;

    let output = reading
        .wifi_2_4_ghz
        .iter()
        .chain(reading.wifi_5_ghz.iter())
        .filter_map(|(_, pair)| {
            pair.iter()
                .filter_map(|(w, s)| {
                    if w.ssid.eq(ssid) && w.mac.eq(mac) {
                        let suggestion = serde_json::to_string(s)
                            .unwrap_or("Couldn't parse the suggestion.".into());
                        Some(RawScan {
                            ssid: Cow::Borrowed(ssid),
                            mac: Cow::Borrowed(mac),
                            suggestion,
                        })
                    } else {
                        None
                    }
                })
                .next()
        })
        .next()?;

    serde_json::to_string(&output).ok()
}

/// Serves a file requested by the user.
#[get("/<id>")]
pub async fn serve(id: ReadingID<'_>) -> Option<File> {
    File::open(id.path()).await.ok()
}

#[get("/")]
pub async fn default_route() -> &'static str {
    "Usage
        Method Route
        GET     /
            - Displays this notice with a explanation of 

        GET     /<file_id>
            - Retrieves, if possible, content from within a user uploaded reading.

        GET     /<ssid>/<mac>
            - Retrieves the latest suggestion available to the given ssid/mac device.

        GET     /<ssid>/<mac>/raw
            - Retrieves the latest suggestion available to the given ssid/mac device in a json format.

        GET     /<file_id>/<ssid>/<mac>
            - Retrieves a suggestion for a specific ssid/mac device in a file specified by the client.
        
        GET     /<file_id>/<ssid>/<mac>/raw
            - Retrieves a suggestion for a specific ssid/mac device in a file specified by the client in a json format.

        POST    /
            - Accepts a json file in the body of the request and responds with a URL leading to the file's content."
}
