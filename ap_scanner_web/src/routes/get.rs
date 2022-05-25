use std::borrow::Cow;

use crate::{Cache, HOST, SCAN_PATH};
use rocket::tokio::fs::File;
use serde::Serialize;
use walkdir::WalkDir;

use crate::readings::ReadingID;

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

#[get("/index")]
pub async fn index() -> String {
    let scans = get_scans();
    let count = scans.len();

    let mut output = String::new();

    output.push_str("AP Scanner Index\n");
    output.push_str(format!("There's currently {} scans available.\n\nScans\n", count).as_str());
    for scan in scans {
        output.push_str(&format!("{host}/{id}\n", host = HOST.as_str(), id = scan))
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

/// Serves a file requested by the user.
///
#[get("/<id>")]
pub async fn serve(id: ReadingID<'_>) -> Option<File> {
    File::open(id.path()).await.ok()
}

#[get("/")]
pub async fn default_route() -> &'static str {
    "Usage

        GET /<file_id>
            - Retrieves, if possible, content from within a user uploaded reading.

        GET /suggestion/<ssid>/<mac>
            - Retrieves the latest suggestion available to the given ssid/mac device.

        POST /
            - Accepts a json file in the body of the request and responds with a URL leading to the file's content."
}
